import { AlertCircle } from 'lucide-react'
import { Card } from '@/components/ui/card'

export default function SilenceWarning() {
  return (
    <Card className="border-amber-700 bg-amber-900/20 p-6 rounded-lg mt-12 mb-8">
      <div className="flex items-start gap-4">
        <AlertCircle className="w-6 h-6 text-amber-500 flex-shrink-0 mt-0.5" />
        <div>
          <p className="font-playfair text-amber-200 font-bold text-lg">
            Silence is never treated as safe.
          </p>
          <p className="text-sm text-amber-100 mt-2">
            The absence of detected risks does not guarantee safety. This analysis reveals known risks and unverified code, 
            but cannot detect novel exploits or attack vectors. Always conduct additional due diligence before interacting with contracts.
          </p>
        </div>
      </div>
    </Card>
  )
}
