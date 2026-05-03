import { IntentResolution } from '@/types/api'
import { Card } from '@/components/ui/card'
import { Copy, Check } from 'lucide-react'
import { useState } from 'react'
import UnscrambleText from './UnscrambleText'

interface IntentResolutionCardProps {
  intentResolution: IntentResolution
}

export default function IntentResolutionCard({ intentResolution }: IntentResolutionCardProps) {
  const [copiedField, setCopiedField] = useState<string | null>(null)

  const handleCopy = (text: string, field: string) => {
    navigator.clipboard.writeText(text)
    setCopiedField(field)
    setTimeout(() => setCopiedField(null), 2000)
  }

  return (
    <Card className="border-slate-800 bg-slate-900/50 p-6 space-y-6">
      <h3 className="text-lg font-playfair text-amber-200">Intent Resolution</h3>

      {/* Normalized To */}
      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-slate-400 mb-2">
          Normalized To
        </label>
        <div className="flex items-center gap-2">
          <code className="flex-1 bg-slate-800 px-3 py-2 rounded text-slate-300 text-sm break-all font-mono">
            {intentResolution.normalized_to}
          </code>
          <button
            onClick={() => handleCopy(intentResolution.normalized_to, 'normalized')}
            className="flex-shrink-0 p-2 hover:bg-slate-800 rounded transition-colors"
          >
            {copiedField === 'normalized' ? (
              <Check className="w-4 h-4 text-emerald-500" />
            ) : (
              <Copy className="w-4 h-4 text-slate-500" />
            )}
          </button>
        </div>
      </div>

      {/* Decoded Call with unscramble animation */}
      <div>
        <label className="block text-xs font-semibold uppercase tracking-wide text-slate-400 mb-2">
          Decoded Call
        </label>
        <code className="block bg-slate-800 px-3 py-3 rounded text-slate-300 text-xs font-mono break-words overflow-auto max-h-32">
          <UnscrambleText text={intentResolution.decoded_call} />
        </code>
      </div>

      {/* Unresolved Legs */}
      {intentResolution.unresolved_legs.length > 0 && (
        <div>
          <label className="block text-xs font-semibold uppercase tracking-wide text-slate-400 mb-2">
            Unresolved Legs
          </label>
          <ul className="space-y-2">
            {intentResolution.unresolved_legs.map((leg, idx) => (
              <li key={idx} className="bg-slate-800 px-3 py-2 rounded text-slate-300 text-sm border border-slate-700">
                {leg}
              </li>
            ))}
          </ul>
        </div>
      )}
    </Card>
  )
}
