import { Leg, VerdictStatus } from '@/types/api'
import { Card } from '@/components/ui/card'
import { CheckCircle2, AlertTriangle, AlertCircle, Copy, Check, ChevronDown, AlertCircleIcon } from 'lucide-react'
import { useState } from 'react'
import ProxyNodeVisualization from './ProxyNodeVisualization'
import BridgeInfoCard from './BridgeInfoCard'
import { truncateAddress } from '@/lib/utils'

interface LegCardProps {
  leg: Leg
  index: number
  totalLegs: number
}

export default function LegCard({ leg, index, totalLegs }: LegCardProps) {
  const [copiedAddress, setCopiedAddress] = useState(false)
  const [expandedBridge, setExpandedBridge] = useState(false)

  const handleCopyAddress = () => {
    navigator.clipboard.writeText(leg.address)
    setCopiedAddress(true)
    setTimeout(() => setCopiedAddress(false), 2000)
  }

  const getVerdictIcon = (verdict: VerdictStatus) => {
    switch (verdict) {
      case 'green':
        return <CheckCircle2 className="w-5 h-5 text-emerald-500" />
      case 'amber':
        return <AlertTriangle className="w-5 h-5 text-amber-500" />
      case 'unverified':
        return <AlertCircle className="w-5 h-5 text-slate-500" />
      case 'red':
        return <AlertCircleIcon className="w-5 h-5 text-rose-500" />
    }
  }

  const getVerdictColor = (verdict: VerdictStatus) => {
    switch (verdict) {
      case 'green':
        return 'border-emerald-600 bg-emerald-900/10'
      case 'amber':
        return 'border-amber-600 bg-amber-900/10'
      case 'unverified':
        return 'border-slate-700 bg-slate-800/30'
      case 'red':
        return 'border-rose-600 bg-rose-900/10'
    }
  }

  const isUnverified = leg.verdict === 'unverified'

  return (
    <Card className={`border-2 ${getVerdictColor(leg.verdict)} p-6 transition-all duration-300 hover:shadow-lg hover:shadow-amber-900/20 ${isUnverified ? 'horizon-grayscale' : ''}`}>
      <div className="space-y-4">
        {/* Header: Address & Verdict */}
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-3 flex-1">
            {getVerdictIcon(leg.verdict)}
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <code className="text-sm font-mono text-amber-200 break-all">
                  {leg.address}
                </code>
                <button
                  onClick={handleCopyAddress}
                  className="flex-shrink-0 p-1.5 hover:bg-slate-800 rounded transition-colors"
                >
                  {copiedAddress ? (
                    <Check className="w-4 h-4 text-emerald-500" />
                  ) : (
                    <Copy className="w-4 h-4 text-slate-500 hover:text-slate-300" />
                  )}
                </button>
              </div>
              <p className="text-xs text-slate-500">
                {leg.chain.toUpperCase()} • Leg {index + 1} of {totalLegs}
              </p>
            </div>
          </div>
          <div className="text-right flex-shrink-0">
            <span className={`inline-block px-3 py-1 rounded text-xs font-semibold ${
              leg.verdict === 'green' ? 'bg-emerald-900/50 text-emerald-200' :
              leg.verdict === 'amber' ? 'bg-amber-900/50 text-amber-200' :
              leg.verdict === 'unverified' ? 'bg-slate-700/50 text-slate-300' :
              'bg-rose-900/50 text-rose-200'
            }`}>
              {leg.verdict.toUpperCase()}
            </span>
          </div>
        </div>

        {/* Source Verification with Reason Code Tooltip */}
        <div className="pt-2 border-t border-slate-700">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full" style={{
              backgroundColor: leg.source_verified ? '#10b981' : '#ef4444'
            }} />
            <span className="text-sm text-slate-300">
              {leg.source_verified ? 'Verified Contract' : 'Unverified Contract'}
            </span>
            {!leg.source_verified && leg.notes.length > 0 && (
              <div className="relative group">
                <AlertCircle className="w-4 h-4 text-amber-500 cursor-help" />
                <div className="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-slate-800 border border-slate-700 rounded p-2 text-xs text-slate-200 whitespace-nowrap z-10">
                  {leg.notes[0]}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Infrastructure Badge */}
        {leg.infra_kind && (
          <div>
            <span className="inline-block px-3 py-1.5 bg-slate-800 border border-slate-700 rounded text-xs font-semibold text-amber-200">
              {leg.infra_kind}
            </span>
          </div>
        )}

        {/* Proxy Node Visualization */}
        {leg.is_proxy && leg.proxy_implementation && (
          <ProxyNodeVisualization proxyAddress={leg.address} implAddress={leg.proxy_implementation} />
        )}

        {/* Notes/Findings */}
        {leg.notes.length > 0 && (
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-slate-400 mb-2">Findings</p>
            <ul className="space-y-1">
              {leg.notes.map((note, idx) => (
                <li key={idx} className="text-sm text-slate-300 flex items-start gap-2">
                  <span className="text-amber-500 mt-0.5">•</span>
                  <span>{note}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Bridge Info */}
        {leg.bridge_info && (
          <div>
            <button
              onClick={() => setExpandedBridge(!expandedBridge)}
              className="w-full flex items-center justify-between p-3 bg-slate-800 hover:bg-slate-700 rounded transition-colors text-left"
            >
              <span className="font-semibold text-amber-200 text-sm">Protocol Health: {leg.bridge_info.protocol}</span>
              <ChevronDown className={`w-4 h-4 text-amber-500 transition-transform ${expandedBridge ? 'rotate-180' : ''}`} />
            </button>
            {expandedBridge && (
              <div className="mt-2">
                <BridgeInfoCard bridgeInfo={leg.bridge_info} />
              </div>
            )}
          </div>
        )}
      </div>
    </Card>
  )
}
