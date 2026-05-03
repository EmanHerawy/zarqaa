import { VerdictStatus } from '@/types/api'
import { AlertCircle, CheckCircle2, AlertTriangle } from 'lucide-react'
import { Card } from '@/components/ui/card'

interface VerdictBannerProps {
  verdict: VerdictStatus
  txHash: string | null
}

export default function VerdictBanner({ verdict, txHash }: VerdictBannerProps) {
  const getVerdictConfig = (v: VerdictStatus) => {
    switch (v) {
      case 'green':
        return {
          icon: CheckCircle2,
          bg: 'bg-emerald-900/20',
          border: 'border-emerald-700',
          textColor: 'text-emerald-200',
          title: 'Verified Safe',
          description: 'All legs are verified and source code is available.',
        }
      case 'amber':
        return {
          icon: AlertTriangle,
          bg: 'bg-amber-900/20',
          border: 'border-amber-700',
          textColor: 'text-amber-200',
          title: 'Review Required',
          description: 'Some legs are unverified or have potential risks.',
        }
      case 'unverified':
        return {
          icon: AlertCircle,
          bg: 'bg-slate-800/50',
          border: 'border-slate-700',
          textColor: 'text-slate-300',
          title: 'Unverified Path',
          description: 'Unable to verify all contracts on this path.',
        }
      case 'red':
        return {
          icon: AlertCircle,
          bg: 'bg-rose-900/20',
          border: 'border-rose-700',
          textColor: 'text-rose-200',
          title: 'High Risk Detected',
          description: 'Critical issues found in transaction path.',
        }
    }
  }

  const config = getVerdictConfig(verdict)
  const Icon = config.icon

  const glowClass = verdict === 'green' ? 'glow-emerald' : verdict === 'amber' ? 'glow-amber' : 'glow-rose'

  return (
    <Card className={`border ${config.border} ${config.bg} p-6 ${glowClass}`}>
      <div className="flex items-start gap-4">
        <Icon className={`w-6 h-6 flex-shrink-0 mt-1 ${config.textColor}`} />
        <div>
          <h2 className={`text-xl font-playfair font-bold ${config.textColor}`}>
            {config.title}
          </h2>
          <p className="text-slate-400 text-sm mt-1">
            {config.description}
          </p>
          {txHash && (
            <p className="text-slate-500 text-xs mt-2 font-mono break-all">
              Hash: {txHash}
            </p>
          )}
        </div>
      </div>
    </Card>
  )
}
