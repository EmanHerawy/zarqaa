'use client'

import Link from 'next/link'
import Image from 'next/image'
import { Button } from '@/components/ui/button'
import { ChevronRight } from 'lucide-react'

export default function LandingPage() {
  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 overflow-hidden">
      {/* Hero Section */}
      <section className="relative min-h-screen flex flex-col items-center justify-center px-4 py-20">
        {/* Background grain effect will be added in Phase 5 */}
        
        <div className="max-w-4xl w-full flex flex-col items-center gap-12 text-center">
          {/* Logo */}
          <div className="relative w-32 h-32 md:w-48 md:h-48">
            <Image
              src="/zarqaa-eye.jpg"
              alt="Zarqaa Eye Logo"
              fill
              className="object-contain drop-shadow-2xl"
              priority
            />
          </div>

          {/* Brand Title */}
          <div className="space-y-2">
            <h1 className="font-playfair text-5xl md:text-7xl font-bold text-amber-200 tracking-wider">
              ZARQAA
            </h1>
            <p className="text-amber-200 text-lg md:text-xl tracking-widest font-light">
              زرقاء
            </p>
          </div>

          {/* Tagline */}
          <p className="text-xl md:text-2xl text-slate-300 font-light max-w-2xl leading-relaxed">
            Transaction-Time Security Intelligence Layer
          </p>

          {/* Legend Story */}
          <div className="max-w-2xl space-y-6 text-left">
            <div className="space-y-4 text-slate-300 leading-relaxed">
              <p className="font-playfair text-2xl text-amber-200 mb-4">
                The Legend of Zarqaa al-Yamama
              </p>
              <p>
                In ancient Arabia, a legendary archer named Zarqaa al-Yamama possessed eyes so keen 
                she could see soldiers hidden miles away—before they revealed themselves. Her vision 
                penetrated deception.
              </p>
              <p>
                Today, Web3 transactions hide their true nature in layers: proxy contracts, 
                abstract intent, and unverified code. <span className="text-amber-200 font-semibold">Hidden soldiers</span> 
                {' '}masquerade as safety.
              </p>
              <p>
                Zarqaa is your digital sight beyond the horizon. It reveals what sleeps in the 
                shadows—not through trust, but through <span className="text-amber-200 font-semibold">forensic clarity</span>.
              </p>
              <p className="italic text-amber-500 border-l-2 border-amber-500 pl-4 py-2">
                "Silence is never treated as safe."
              </p>
            </div>
          </div>

          {/* CTA Button */}
          <Link href="/scanner" className="w-full max-w-sm">
            <Button 
              size="lg" 
              className="w-full bg-amber-500 hover:bg-amber-600 text-slate-950 font-bold text-lg py-6 rounded-lg transition-all duration-300 flex items-center justify-center gap-2 group"
            >
              Scan A Transaction
              <ChevronRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
            </Button>
          </Link>

          {/* Secondary CTA */}
          <p className="text-slate-400 text-sm">
            Or analyze an intent to see pre-transaction risks.
          </p>
        </div>
      </section>

      {/* Footer message */}
      <section className="border-t border-slate-800 py-8 text-center text-slate-500 text-sm">
        <p>Zarqaa reveals what others cannot see. Trust through transparency.</p>
      </section>
    </main>
  )
}
