import { MEVRisk } from '@/types/api'
import { Card } from '@/components/ui/card'
import { AlertTriangle, ExternalLink, Radar } from 'lucide-react'

interface MEVRiskCardProps {
  mevRisk: MEVRisk
}

export default function MEVRiskCard({ mevRisk }: MEVRiskCardProps) {
  const isHighRisk = mevRisk.risk_level === 'high'

  return (
    <Card className={`border-2 ${isHighRisk ? 'border-rose-600 bg-rose-900/20' : 'border-amber-600 bg-amber-900/20'} p-6 relative overflow-hidden`}>
      {/* MEV Threat Radar (for high-risk) */}
      {isHighRisk && (
        <div className="absolute top-4 right-4 w-20 h-20">
          <svg className="w-full h-full animate-pulse" viewBox="0 0 100 100">
            {/* Radar circles */}
            <circle cx="50" cy="50" r="45" fill="none" stroke="#f87171" strokeWidth="0.5" opacity="0.3" />
            <circle cx="50" cy="50" r="30" fill="none" stroke="#f87171" strokeWidth="0.5" opacity="0.5" />
            <circle cx="50" cy="50" r="15" fill="none" stroke="#f87171" strokeWidth="0.5" opacity="0.7" />
            {/* Center ping */}
            <circle cx="50" cy="50" r="4" fill="#f87171" />
            {/* Radar sweep lines */}
            <line x1="50" y1="50" x2="50" y2="10" stroke="#f87171" strokeWidth="0.5" opacity="0.6" />
            <line x1="50" y1="50" x2="85" y2="50" stroke="#f87171" strokeWidth="0.5" opacity="0.6" />
            <line x1="50" y1="50" x2="50" y2="90" stroke="#f87171" strokeWidth="0.5" opacity="0.6" />
          </svg>
        </div>
      )}

      <div className="space-y-4 relative z-10">
        <div className="flex items-start gap-3">
          {isHighRisk ? (
            <Radar className={`w-6 h-6 flex-shrink-0 mt-0.5 ${isHighRisk ? 'text-rose-400 animate-pulse' : 'text-amber-400'}`} />
          ) : (
            <AlertTriangle className={`w-6 h-6 flex-shrink-0 mt-0.5 ${isHighRisk ? 'text-rose-400' : 'text-amber-400'}`} />
          )}
          <div>
            <h3 className={`font-playfair font-bold text-lg ${isHighRisk ? 'text-rose-200' : 'text-amber-200'}`}>
              MEV Risk: {mevRisk.risk_level.toUpperCase()}
            </h3>
            <p className="text-slate-300 text-sm mt-2">
              {mevRisk.summary}
            </p>
          </div>
        </div>

        {/* Signals */}
        {mevRisk.signals.length > 0 && (
          <div>
            <p className="text-slate-400 text-xs font-semibold uppercase tracking-wide mb-2">Detected Signals</p>
            <div className="space-y-1">
              {mevRisk.signals.map((signal, idx) => (
                <p key={idx} className="text-slate-300 text-sm">
                  • {signal}
                </p>
              ))}
            </div>
          </div>
        )}

        {/* Recommendation */}
        <div className="mt-4 pt-4 border-t border-slate-700">
          <p className="text-slate-400 text-xs font-semibold uppercase tracking-wide mb-2">Recommendation</p>
          <p className="text-slate-300 text-sm">
            {mevRisk.recommendation}
          </p>
          
          {/* Extract and render links from recommendation */}
          {mevRisk.recommendation.includes('http') && (
            <div className="flex gap-3 mt-3">
              {mevRisk.recommendation.includes('flashbots') && (
                <a 
                  href="https://protect.flashbots.net" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-amber-400 hover:text-amber-300 text-sm"
                >
                  Flashbots Protect
                  <ExternalLink className="w-3 h-3" />
                </a>
              )}
              {mevRisk.recommendation.includes('mevblocker') && (
                <a 
                  href="https://mevblocker.io" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-amber-400 hover:text-amber-300 text-sm"
                >
                  MEV Blocker
                  <ExternalLink className="w-3 h-3" />
                </a>
              )}
            </div>
          )}
        </div>
      </div>
    </Card>
  )
}
