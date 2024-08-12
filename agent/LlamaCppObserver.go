package agent

import (
	"context"
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/management"
	"github.com/google/uuid"
	"github.com/hashicorp/go-hclog"
)

type LlamaCppObserver struct {
	AgentConfiguration            *AgentConfiguration
	ExternalLlamaCppConfiguration *llamacpp.LlamaCppConfiguration
	LlamaCppClient                *llamacpp.LlamaCppClient
	Logger                        hclog.Logger
	ManagementClient              *management.Client
}

func (self *LlamaCppObserver) ObserveAndReport(
	serverEventsChannel chan<- goroutine.ResultMessage,
) {
	self.Logger.Debug(
		"observing",
		"host", self.LlamaCppClient.LlamaCppConfiguration.HttpAddress.GetHostWithPort(),
	)

	agentRuntimeId, err := uuid.NewV7()

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to generate agent uuid",
			Error:   err,
		}

		return
	}

	agentRuntimeIdString := agentRuntimeId.String()

	llamaCppHealthStatusChannel := make(chan llamacpp.LlamaCppHealthStatus)

	defer close(llamaCppHealthStatusChannel)

	ticker := time.NewTicker(self.AgentConfiguration.GetReportingIntervalDuration())

	go self.RunTickerInterval(llamaCppHealthStatusChannel, ticker)

	for llamaCppHealthStatus := range llamaCppHealthStatusChannel {
		ctx, cancel := context.WithTimeout(
			context.Background(),
			self.AgentConfiguration.GetReportingIntervalDuration(),
		)

		self.ManagementClient.ReportLlamaCppHealthStatus(
			ctx,
			serverEventsChannel,
			self.ExternalLlamaCppConfiguration,
			&llamaCppHealthStatus,
			agentRuntimeIdString,
			self.AgentConfiguration.Name,
		)

		cancel()
	}
}

func (self *LlamaCppObserver) RunTickerInterval(
	llamaCppHealthStatusChannel chan llamacpp.LlamaCppHealthStatus,
	ticker *time.Ticker,
) {
	for range ticker.C {
		ctx, cancel := context.WithTimeout(
			context.Background(),
			self.AgentConfiguration.GetReportingIntervalDuration(),
		)

		self.LlamaCppClient.GetHealth(
			ctx,
			llamaCppHealthStatusChannel,
		)

		cancel()
	}
}
