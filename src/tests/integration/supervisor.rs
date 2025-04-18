#[cfg(test)]
mod tests {
    use cucumber::{given, then, when, World};
    use serde_json::{json, Value};

    use crate::{
        balancer::upstream_peer_pool::UpstreamPeerPool,
        errors::result::Result,
        tests::integration::utils::utils::{
            get_unix_time_from, start_prometheus, start_statsd, start_supervisor, PaddlerWorld,
        },
    };

    use std::{net::SocketAddr, process::Command, str::FromStr};

    #[given(
        expr = "{word} is running at {word}, {word} and reports metrics to {word} every {int} second(s)"
    )]
    async fn balancer_is_running(
        world: &mut PaddlerWorld,
        _balancer_name: String,
        management_addr: String,
        reveseproxy_addr: String,
        statsd_addr: String,
        reporting_interval: usize,
    ) -> Result<()> {
        world.balancer1 = Some(
            Command::new("target/release/paddler")
                .args([
                    "balancer",
                    "--management-addr",
                    &management_addr,
                    "--reverseproxy-addr",
                    &reveseproxy_addr,
                    "--statsd-addr",
                    &statsd_addr,
                    "--statsd-reporting-interval",
                    &reporting_interval.to_string(),
                    "--management-dashboard-enable",
                ])
                .spawn()
                .expect("Failed to run balancer"),
        );

        Ok(())
    }

    #[given(expr = "{word} is running at {word}, {word} and receives metrics from {word}")]
    async fn statsd_is_running(
        world: &mut PaddlerWorld,
        _statsd_name: String,
        exporter_addr: String,
        management_addr: String,
    ) -> Result<()> {
        world.statsd = Some(start_statsd(management_addr, exporter_addr)?);

        Ok(())
    }

    #[given(
        expr = "{word} is running at {word} and scrapes metrics from {word} every {int} second(s)"
    )]
    async fn prometheus_is_running(
        world: &mut PaddlerWorld,
        _prometheus_name: String,
        prometheus_addr: String,
        statsd_addr: String,
        _monitoring_interval: usize,
    ) -> Result<()> {
        world.prometheus = Some(start_prometheus(prometheus_addr, statsd_addr)?);

        Ok(())
    }

    #[given(
        expr = "{word} is running at {word} with {word} configuration stored on {word} and starts {word} at {int} with {int} slot(s) running {word}"
    )]
    async fn supervisor_is_running(
        world: &mut PaddlerWorld,
        supervisor_name: String,
        supervisor_addr: String,
        driver_type: String,
        driver_addr: String,
        _llamacpp_name: String,
        llamacpp_addr: String,
        _slots: usize,
        model_name: String,
    ) -> Result<()> {
        match supervisor_name.as_str() {
            "supervisor-1" => {
                world.supervisor1 = Some(start_supervisor(
                    supervisor_name,
                    supervisor_addr,
                    driver_type,
                    driver_addr,
                    llamacpp_addr,
                    model_name,
                )?)
            }
            "supervisor-2" => {
                world.supervisor2 = Some(start_supervisor(
                    supervisor_name,
                    supervisor_addr,
                    driver_type,
                    driver_addr,
                    llamacpp_addr,
                    model_name,
                )?)
            }
            _ => (),
        }

        Ok(())
    }

    #[when(
        expr = "{word} is running and observing {word} in {word}, and registered at {word} in {word}"
    )]
    async fn agent_is_running(
        world: &mut PaddlerWorld,
        agent_name: String,
        _llamacpp_name: String,
        llamacpp_addr: String,
        _balancer_name: String,
        balancer_addr: String,
    ) -> Result<()> {
        let process = Some(
            Command::new("target/release/paddler")
                .args([
                    "agent",
                    "--local-llamacpp-addr",
                    &llamacpp_addr,
                    "--management-addr",
                    &balancer_addr,
                    "--name",
                    &agent_name,
                ])
                .spawn()
                .expect("Failed to run balancer"),
        );

        match agent_name.as_str() {
            "agent-1" => world.agent1 = process,
            "agent-2" => world.agent2 = process,
            _ => (),
        }

        Ok(())
    }

    #[then(
        expr = "{word} in {word} must report that {word} is registered with {int} slots at {word}"
    )]
    async fn display_agent_slots(
        _world: &mut PaddlerWorld,
        _balancer_name: String,
        balancer_addr: String,
        agent_name: String,
        slots_idle: usize,
        llamacpp_addr: String,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(3));

        let mut response = serde_json::from_str::<UpstreamPeerPool>(
            &reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
                .await?
                .text()
                .await?,
        )?;

        let agents = response.agents.get_mut()?;

        let agent = agents
            .into_iter()
            .find(|agent| agent.agent_name == Some(agent_name.to_string()));

        if let Some(agent) = agent {
            assert_eq!(agent.slots_idle, slots_idle);
            assert_eq!(agent.slots_processing, 0);
            assert_eq!(agent.error, None);
            assert_eq!(
                agent.external_llamacpp_addr,
                SocketAddr::from_str(&llamacpp_addr)?
            );
            assert_eq!(agent.is_authorized, Some(true));
            assert_eq!(agent.is_slots_endpoint_enabled, Some(true));
        }

        Ok(())
    }

    #[when(expr = r"{int} request(s) is/are proxied to {word} in {word}")]
    async fn proxy_balancer(
        _world: &mut PaddlerWorld,
        requests: usize,
        _balancer_name: String,
        balancer_addr: String,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(15));

        let client = reqwest::Client::new();

        let value = json!({
            "model": "qwen2_500m.gguf",
            "messages": [
                {
                    "role": "user",
                    "content": "List all prime numbers between 10,000 and 20,000,
                    verifying what are possible calculable primes by Lucas-Lehmer
                    test. Format as a numbered list with the verification proof
                    for each entry. And tell a story
                    about each number."
                }
            ]
        });

        let mut handles = vec![];

        for _ in 0..requests {
            let client = client.clone();
            let addr = balancer_addr.clone();
            let value = value.clone();
            handles.push(tokio::spawn(async move {
                client
                    .post(format!("http://{}/chat/completions", addr))
                    .json(&value)
                    .send()
                    .await
            }));
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        Ok(())
    }

    #[when(
        expr = r"{int} request(s) is/are proxied to {word} in {word} to change slots to {int} and port to {int}"
    )]
    async fn proxy_supervisor(
        _world: &mut PaddlerWorld,
        requests: usize,
        _supervisor_name: String,
        supervisor_addr: String,
        slots: usize,
        port: usize,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(5));

        let client = reqwest::Client::new();

        let value = json!(
        {
            "args": {
                "-m": "qwen2_500m.gguf",
                "--port": port,
                "binary": "llama-server",
                "-np": slots,
                "--slots": ""
            }
        }
        );

        let mut handles = vec![];

        for _ in 0..requests {
            let client = client.clone();
            let value = value.clone();
            let addr = supervisor_addr.clone();
            handles.push(tokio::spawn(async move {
                client
                    .post(format!("http://{}/v1/params", addr))
                    .json(&value)
                    .send()
                    .await
                    .unwrap();
            }));
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        Ok(())
    }

    #[then(
        expr = "{word} must tell {int} slot(s) is/are busy and {int} slot(s) is/are idle in {word} from {word} and {word}"
    )]
    async fn slot_is_busy(
        _world: &mut PaddlerWorld,
        _balancer_name: String,
        slots_busy: usize,
        idle_slots: usize,
        balancer_addr: String,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(2));

        let response = serde_json::from_str::<UpstreamPeerPool>(
            reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
                .await?
                .text()
                .await?
                .as_str(),
        )?;

        let (slots_idle, slots_processing) = response.total_slots()?;

        assert_eq!(idle_slots, slots_idle);
        assert_eq!(slots_processing, slots_busy);

        Ok(())
    }

    #[then(expr = r"{word} must return a(n) {word} response in {word}")]
    async fn get_response(
        world: &mut PaddlerWorld,
        _balancer_name: String,
        _balancer_addr: String,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(7));

        for response in &world.proxy_response {
            if let Some(response) = response {
                assert!(response.is_ok());
            }
        }

        world.proxy_response.clear();

        Ok(())
    }

    #[then(expr = "{word} must tell {int} slot(s) is/are {word} at {word} from {word}")]
    async fn report_metrics(
        _world: &mut PaddlerWorld,
        _statsd_name: String,
        slots: usize,
        state: String,
        prometheus_addr: String,
    ) -> Result<()> {
        let start = get_unix_time_from(0);
        let end = get_unix_time_from(15);
        let step = 5;

        std::thread::sleep(std::time::Duration::from_millis(2000));

        let response = reqwest::get(format!(
            "http://{}/api/v1/query?query=paddler_slots_{}&start={}&end={}&step={}",
            prometheus_addr, state, start, end, step
        ))
        .await?
        .text()
        .await?;

        let v: Value = serde_json::from_str(&response).unwrap();

        if let Some(metrics) = v.as_object() {
            if let Some(data) = metrics.get("data") {
                if let Some(result) = data.get("result") {
                    if let Some(value) = result[0].get("value") {
                        assert_eq!(value[1].as_str(), Some(slots.to_string().as_str()))
                    }
                }
            }
        }

        Ok(())
    }

    #[tokio::test]
    pub async fn run_cucumber_tests() {
        PaddlerWorld::cucumber()
            .max_concurrent_scenarios(1)
            .fail_fast()
            // .retries(3)
            // .retry_after(std::time::Duration::from_secs(60))
            .fail_on_skipped()
            .before(|_feature, _rule, _scenario, world| {
                Box::pin(async move {
                    world.setup().expect("Setup Failed");
                })
            })
            .after(|_feature, _rule, _scenario, _scenario_finished, world| {
                Box::pin(async move {
                    if let Some(world) = world {
                        world.teardown().await.expect("Teardown Failed");
                    }
                })
            })
            .run("src/tests/integration/features/supervisor.feature")
            .await;
    }
}
