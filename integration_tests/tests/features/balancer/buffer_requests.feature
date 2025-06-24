Feature: Buffer llama.cpp requests

    Background:
        Given balancer is running (8 max requests)

    @serial
    Scenario: More requests than slots available
        Given llama.cpp server "llama-1" is running (has 1 slot)
        Given llama.cpp server "llama-2" is running (has 1 slot)
        Given agent "agent-1" is running (observes "llama-1")
        Given agent "agent-1" is registered
        Given agent "agent-2" is running (observes "llama-2")
        Given agent "agent-2" is registered
        When multiple requests are sent to "/chat/completions"
            | req-1 |
            | req-2 |
            | req-3 |
            | req-4 |
            | req-5 |
            | req-6 |
        Then "req-1" response code is 200
        Then "req-1" request landed in "llama-1"
        Then "req-2" response code is 200
        Then "req-2" request landed in "llama-2"
        Then "req-3" response code is 200
        Then "req-3" request landed in "llama-3"
        Then "req-4" response code is 200
        Then "req-4" request landed in "llama-4"
        Then "req-5" response code is 200
        Then "req-5" request landed in "llama-5"
        Then "req-6" response code is 200
        Then "req-6" request landed in "llama-6"
