import { Copy, Check } from 'lucide-react'
import { useState } from 'react'
import { truncateAddress } from '@/lib/utils'

interface ProxyNodeVisualizationProps {
  proxyAddress: string
  implAddress: string
}

export default function ProxyNodeVisualization({ proxyAddress, implAddress }: ProxyNodeVisualizationProps) {
  const [copiedImpl, setCopiedImpl] = useState(false)

  const handleCopyImpl = () => {
    navigator.clipboard.writeText(implAddress)
    setCopiedImpl(true)
    setTimeout(() => setCopiedImpl(false), 2000)
  }

  return (
    <div className="space-y-3 p-3 bg-slate-800/50 rounded border border-slate-700">
      <p className="text-xs font-semibold uppercase tracking-wide text-slate-400">Proxy Architecture</p>
      
      {/* Visual Connection */}
      <div className="space-y-2">
        {/* Proxy Node */}
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-amber-500" />
          <span className="text-xs text-slate-400">Proxy Contract</span>
        </div>

        {/* Connection Line */}
        <div className="ml-1.5 border-l-2 border-slate-700 h-2" />

        {/* Implementation Node */}
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-slate-600" />
          <span className="text-xs text-slate-400">Implementation</span>
        </div>
      </div>

      {/* Implementation Address */}
      <div className="mt-3 p-2 bg-slate-900 rounded border border-slate-800">
        <p className="text-xs text-slate-500 mb-1">Implementation Address</p>
        <div className="flex items-center gap-2">
          <code className="text-xs font-mono text-slate-300 flex-1 break-all">
            {implAddress}
          </code>
          <button
            onClick={handleCopyImpl}
            className="flex-shrink-0 p-1 hover:bg-slate-700 rounded transition-colors"
          >
            {copiedImpl ? (
              <Check className="w-3 h-3 text-emerald-500" />
            ) : (
              <Copy className="w-3 h-3 text-slate-500 hover:text-slate-300" />
            )}
          </button>
        </div>
      </div>

      <p className="text-xs text-slate-500">
        This contract uses a transparent proxy pattern (ERC-1967) with a separate implementation.
      </p>
    </div>
  )
}
