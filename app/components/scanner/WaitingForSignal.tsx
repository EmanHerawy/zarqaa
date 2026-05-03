import { Eye } from 'lucide-react'

export default function WaitingForSignal() {
  return (
    <div className="flex flex-col items-center justify-center py-20 space-y-6">
      <div className="relative w-24 h-24 opacity-30">
        <Eye className="w-full h-full text-amber-500" strokeWidth={1} />
      </div>
      <div className="text-center space-y-2">
        <p className="text-slate-400 text-lg font-light">Waiting for signal...</p>
        <p className="text-slate-600 text-sm">
          Enter a transaction hash or intent to begin your scan
        </p>
      </div>
    </div>
  )
}
