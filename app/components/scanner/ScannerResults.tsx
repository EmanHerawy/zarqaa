import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import { AnalysisResponse } from '@/types/api'
import AnimatedVerdictBanner from './AnimatedVerdictBanner'
import MEVRiskCard from './MEVRiskCard'
import IntentResolutionCard from './IntentResolutionCard'
import AnimatedLegCard from './AnimatedLegCard'
import FlightPathVisualization from './FlightPathVisualization'
import SilenceWarning from './SilenceWarning'

interface ScannerResultsProps {
  results: AnalysisResponse
  onBack: () => void
}

export default function ScannerResults({ results, onBack }: ScannerResultsProps) {
  return (
    <div className="space-y-8 pb-12">
      {/* Back Button */}
      <Button
        onClick={onBack}
        variant="ghost"
        className="text-amber-500 hover:text-amber-400 hover:bg-transparent"
      >
        <ArrowLeft className="w-4 h-4 mr-2" />
        New Scan
      </Button>

      {/* Verdict Banner */}
      <AnimatedVerdictBanner verdict={results.route_verdict} txHash={results.tx_hash} />

      {/* MEV Risk — intent-based only, medium/high only */}
      {results.intent_resolution &&
        results.mev_risk &&
        (results.mev_risk.risk_level === 'medium' || results.mev_risk.risk_level === 'high') && (
        <MEVRiskCard mevRisk={results.mev_risk} />
      )}

      {/* Intent Resolution (for intent) */}
      {results.intent_resolution && (
        <IntentResolutionCard intentResolution={results.intent_resolution} />
      )}

      {/* Flight Path Visualization */}
      <FlightPathVisualization legs={results.legs} />

      {/* Leg Cards */}
      <div className="space-y-4">
        <h2 className="text-2xl font-playfair text-amber-200">Flight Path Details</h2>
        <div className="space-y-4">
          {results.legs.map((leg, index) => (
            <AnimatedLegCard key={leg.address} leg={leg} index={index} totalLegs={results.legs.length} />
          ))}
        </div>
      </div>

      {/* Silence Warning Footer */}
      <SilenceWarning />
    </div>
  )
}
