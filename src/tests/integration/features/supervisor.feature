Feature: Supervisor

    Scenario: Supervisor can start llamacpp and be registered in balancer
      Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second
      Given supervisor-1 is running at 0.0.0.0:8087 with file configuration stored on supervisor-1.toml and starts llamacpp-1 at 8080 with 4 slots running qwen2_500m.gguf
      Given supervisor-2 is running at 0.0.0.0:8088 with file configuration stored on supervisor-2.toml and starts llamacpp-2 at 8081 with 4 slots running qwen2_500m.gguf

      When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080

      When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070
      Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 4 slots at 0.0.0.0:8081

    # Scenario: Supervisor can start llamacpp and change its arguments
    #   Given balancer-1 is running at 0.0.0.0:8070, 0.0.0.0:8071 and reports metrics to 0.0.0.0:9125 every 1 second
    #   Given supervisor-1 is running at 0.0.0.0:8087 with file configuration stored on supervisor-1.toml and starts llamacpp-1 at 8080 with 4 slots running qwen2_500m.gguf
    #   Given supervisor-2 is running at 0.0.0.0:8088 with file configuration stored on supervisor-2.toml and starts llamacpp-2 at 8081 with 4 slots running qwen2_500m.gguf

    #   When agent-1 is running and observing llamacpp-1 in 0.0.0.0:8080, and registered at balancer-1 in 0.0.0.0:8070
    #   Then balancer-1 in 0.0.0.0:8070 must report that agent-1 is registered with 4 slots at 0.0.0.0:8080

    #   When agent-2 is running and observing llamacpp-2 in 0.0.0.0:8081, and registered at balancer-1 in 0.0.0.0:8070
    #   Then balancer-1 in 0.0.0.0:8070 must report that agent-2 is registered with 4 slots at 0.0.0.0:8081

    #   When 1 request is proxied to supervisor-1 in 0.0.0.0:8087 to change slots to 2
    #   Then balancer-1 must tell 6 slots are busy and 0 slots are idle in 127.0.0.1:8070 from agent-1 and agent-2
    