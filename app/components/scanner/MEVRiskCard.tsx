import { MEVRisk } from '@/types/api'
import { Card } from '@/components/ui/card'
import { AlertTriangle, ExternalLink, Radar } from 'lucide-react'

interface MEVRiskCardProps {
  mevRisk: MEVRisk
}

export default function MEVRiskCard({ mevRisk }: MEVRiskCardProps) {
  const level = mevRisk.risk_level // 'low' | 'medium' | 'high'
  const isHigh = level === 'high'
  const isMedium = level === 'medium'

  // Don't render for low/none — caller should also gate on this,
  // but guard here too so the component is self-contained.
  if (!isHigh && !isMedium) return null

  return (
    <Card
      className={`p-6 relative overflow-hidden border ${
        isHigh
          ? 'border-rose-700/60 bg-rose-900/15'
          : 'border-amber-700/40 bg-amber-900/10'
      }`}
    >
      {/* Animated radar — high risk only */}
      {isHigh && (
        <div className="absolute top-4 right-4 w-16 h-16 opacity-40">
          <svg className="w-full h-full animate-pulse" viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="45" fill="none" stroke="#f87171" strokeWidth="0.8" opacity="0.3" />
            <circle cx="50" cy="50" r="28" fill="none" stroke="#f87171" strokeWidth="0.8" opacity="0.5" />
            <circle cx="50" cy="50" r="12" fill="none" stroke="#f87171" strokeWidth="0.8" opacity="0.7" />
            <circle cx="50" cy="50" r="3" fill="#f87171" />
          </svg>
        </div>
      )}

      <div className="space-y-4 relative z-10">
        {/* Header */}
        <div className="flex items-start gap-3">
          {isHigh
            ? <Radar className="w-5 h-5 flex-shrink-0 mt-0.5 text-rose-400 animate-pulse" />
            : <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5 text-amber-400" />
          }
          <div>
            <h3 className={`font-semibold text-base ${isHigh ? 'text-rose-300' : 'text-amber-300'}`}>
              {isHigh ? 'High MEV Exposure' : 'Potential MEV Exposure'}
            </h3>
            <p className="text-slate-400 text-sm mt-1">
              {mevRisk.summary}
            </p>
          </div>
        </div>

        {/* Mempool context note — intent-based pre-sign context */}
        <p className="text-slate-400 text-sm border-l-2 border-slate-600 pl-3 italic">
          {isHigh
            ? 'This transaction is likely to attract sandwich bots if broadcast to the public mempool. Whether it gets attacked depends on which RPC endpoint you use.'
            : 'This might happen depending on which mempool you route through. Public RPC endpoints expose your transaction to searchers before it lands on-chain.'
          }
        </p>

        {/* Signals */}
        {mevRisk.signals.length > 0 && (
          <div>
            <p className="text-slate-500 text-xs font-semibold uppercase tracking-wide mb-1.5">Detected signals</p>
            <div className="flex flex-wrap gap-2">
              {mevRisk.signals.map((signal, idx) => (
                <span
                  key={idx}
                  className={`text-xs px-2 py-0.5 rounded-full border ${
                    isHigh
                      ? 'border-rose-700/50 bg-rose-900/20 text-rose-300'
                      : 'border-amber-700/40 bg-amber-900/15 text-amber-300'
                  }`}
                >
                  {signal}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Recommendation + links */}
        <div className="pt-3 border-t border-slate-800">
          <p className="text-slate-300 text-sm">{mevRisk.recommendation}</p>

          {mevRisk.recommendation.includes('http') && (
            <div className="flex gap-4 mt-3">
              {mevRisk.recommendation.toLowerCase().includes('flashbots') && (
                <a
                  href="https://protect.flashbots.net"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-amber-400 hover:text-amber-300 text-sm"
                >
                  Flashbots Protect <ExternalLink className="w-3 h-3" />
                </a>
              )}
              {mevRisk.recommendation.toLowerCase().includes('mevblocker') && (
                <a
                  href="https://mevblocker.io"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-amber-400 hover:text-amber-300 text-sm"
                >
                  MEV Blocker <ExternalLink className="w-3 h-3" />
                </a>
              )}
            </div>
          )}
        </div>
      </div>
    </Card>
  )
}
