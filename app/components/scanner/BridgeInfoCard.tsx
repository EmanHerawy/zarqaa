import { BridgeInfo } from '@/types/api'
import { Card } from '@/components/ui/card'
import { AlertCircle, Shield, TrendingUp, CheckCircle2 } from 'lucide-react'

interface BridgeInfoCardProps {
  bridgeInfo: BridgeInfo
}

export default function BridgeInfoCard({ bridgeInfo }: BridgeInfoCardProps) {
  return (
    <Card className="border-slate-700 bg-slate-900/50 p-6 space-y-4">
      {/* Protocol Summary */}
      <div className="space-y-2">
        <p className="text-amber-200 font-semibold">{bridgeInfo.protocol}</p>
        <p className="text-sm text-slate-300">{bridgeInfo.summary}</p>
      </div>

      {/* Key Metrics Grid */}
      <div className="grid grid-cols-2 gap-4 pt-2 border-t border-slate-800">
        {/* DVN Count */}
        <div>
          <p className="text-xs text-slate-500 uppercase tracking-wide mb-1">Verifiers (DVN)</p>
          <p className="text-lg font-bold text-amber-200">{bridgeInfo.dvn_count}</p>
        </div>

        {/* Past Exploits */}
        <div>
          <p className="text-xs text-slate-500 uppercase tracking-wide mb-1">Past Exploits</p>
          <p className={`text-lg font-bold ${bridgeInfo.past_exploits_usd > 0 ? 'text-rose-400' : 'text-emerald-400'}`}>
            ${bridgeInfo.past_exploits_usd > 0 ? `${(bridgeInfo.past_exploits_usd / 1_000_000).toFixed(1)}M` : '0'}
          </p>
        </div>

        {/* Centralization Risk */}
        <div>
          <p className="text-xs text-slate-500 uppercase tracking-wide mb-1">Centralization Risk</p>
          <p className={`text-sm font-semibold ${
            bridgeInfo.centralization_risk === 'low' ? 'text-emerald-400' :
            bridgeInfo.centralization_risk === 'medium' ? 'text-amber-400' :
            'text-rose-400'
          }`}>
            {bridgeInfo.centralization_risk.toUpperCase()}
          </p>
        </div>

        {/* Controller Type */}
        <div>
          <p className="text-xs text-slate-500 uppercase tracking-wide mb-1">Controller</p>
          <p className="text-sm font-semibold text-slate-300">
            {bridgeInfo.controller_type.charAt(0).toUpperCase() + bridgeInfo.controller_type.slice(1)}
          </p>
        </div>
      </div>

      {/* Safety Features */}
      <div className="grid grid-cols-2 gap-2 pt-2 border-t border-slate-800">
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${bridgeInfo.has_rate_limits ? 'bg-emerald-500' : 'bg-slate-600'}`} />
          <span className="text-xs text-slate-400">Rate Limits: {bridgeInfo.has_rate_limits ? 'Yes' : 'No'}</span>
        </div>
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${bridgeInfo.has_circuit_breaker ? 'bg-emerald-500' : 'bg-slate-600'}`} />
          <span className="text-xs text-slate-400">Circuit Breaker: {bridgeInfo.has_circuit_breaker ? 'Yes' : 'No'}</span>
        </div>
      </div>

      {/* Past Exploit Note */}
      {bridgeInfo.past_exploit_note && (
        <div className="p-3 bg-amber-900/20 border border-amber-700 rounded flex gap-3">
          <AlertCircle className="w-4 h-4 text-amber-400 flex-shrink-0 mt-0.5" />
          <p className="text-xs text-amber-200">{bridgeInfo.past_exploit_note}</p>
        </div>
      )}

      {/* Verdict */}
      <div className="p-3 bg-slate-800/50 rounded border border-slate-700">
        <p className="text-xs font-semibold uppercase tracking-wide text-slate-400 mb-1">Verdict</p>
        <p className="text-sm text-slate-300">{bridgeInfo.verdict_summary}</p>
      </div>

      {/* Security Info */}
      <div className="space-y-2 pt-2 border-t border-slate-800">
        <p className="text-xs font-semibold uppercase tracking-wide text-slate-400">Security & Audits</p>
        <div className="space-y-1 text-xs text-slate-400">
          <p>Last Audit: {bridgeInfo.last_audit}</p>
          <p>Bug Bounty: ${(bridgeInfo.bug_bounty_usd / 1_000_000).toFixed(0)}M</p>
          <p>Upgrade Timelock: {bridgeInfo.upgrade_timelock_days ? `${bridgeInfo.upgrade_timelock_days} days` : 'None'}</p>
        </div>
      </div>
    </Card>
  )
}
