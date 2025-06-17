mod tests;
mod outils;

use crate::balancer::balancer_world::BalancerWorld;

#[tokio::main]
async fn main() {
    BalancerWorld::cucumber()
        .after(|_feature, _rule, _scenario, _ev, world| {
            Box::pin(async move {
                world.unwrap().cleanup().await;
            })
        })
        .fail_on_skipped()
        .run("tests/features/balancer")
        .await;
}
