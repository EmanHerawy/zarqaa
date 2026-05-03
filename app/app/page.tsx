'use client'

import Link from 'next/link'
import Image from 'next/image'
import { Button } from '@/components/ui/button'
import { ChevronRight, Shield, Eye, Zap, AlertTriangle } from 'lucide-react'

export default function LandingPage() {
  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 overflow-hidden">

      {/* Hero */}
      <section className="relative min-h-screen flex flex-col items-center justify-center px-4 py-20">

        {/* Ambient glow behind logo */}
        <div className="absolute top-1/3 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-amber-500/10 rounded-full blur-3xl pointer-events-none" />

        <div className="relative max-w-4xl w-full flex flex-col items-center gap-10 text-center">

          {/* Logo */}
          <div className="relative w-40 h-40 md:w-56 md:h-56 drop-shadow-[0_0_40px_rgba(245,158,11,0.35)]">
            <Image
              src="/zarqaa-logo.png"
              alt="Zarqaa"
              fill
              className="object-contain"
              priority
            />
          </div>

          {/* Title */}
          <div className="space-y-1">
            <h1 className="font-playfair text-6xl md:text-8xl font-bold text-amber-200 tracking-widest">
              ZARQAA
            </h1>
            <p className="text-amber-500/80 text-xl md:text-2xl tracking-[0.4em] font-light">
              زرقاء
            </p>
          </div>

          {/* Tagline */}
          <p className="text-2xl md:text-3xl text-slate-200 font-light max-w-2xl leading-snug">
            See every contract your transaction will touch —{' '}
            <span className="text-amber-400 font-semibold">before you sign.</span>
          </p>

          {/* Feature pills */}
          <div className="flex flex-wrap justify-center gap-3">
            {[
              { icon: Eye,          label: 'Full call-path resolution' },
              { icon: Shield,       label: 'Per-contract trust scores' },
              { icon: AlertTriangle,label: 'MEV & sandwich detection'  },
              { icon: Zap,          label: 'Pre-sign + post-submit'     },
            ].map(({ icon: Icon, label }) => (
              <span
                key={label}
                className="flex items-center gap-2 px-4 py-2 rounded-full border border-amber-500/30 bg-amber-500/5 text-amber-200 text-sm"
              >
                <Icon className="w-4 h-4 text-amber-500" />
                {label}
              </span>
            ))}
          </div>

          {/* CTA */}
          <Link href="/scanner" className="w-full max-w-sm mt-2">
            <Button
              size="lg"
              className="w-full bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold text-lg py-6 rounded-xl transition-all duration-200 flex items-center justify-center gap-2 group shadow-[0_0_30px_rgba(245,158,11,0.3)]"
            >
              Scan a Transaction
              <ChevronRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
            </Button>
          </Link>

          <p className="text-slate-500 text-sm">
            Paste a tx hash or describe your intent — results in seconds.
          </p>
        </div>
      </section>

      {/* Legend */}
      <section className="border-t border-slate-800/60 bg-slate-900/40 py-20 px-4">
        <div className="max-w-3xl mx-auto space-y-8">

          <div className="text-center space-y-2">
            <p className="text-amber-500 text-sm tracking-widest uppercase">The Legend</p>
            <h2 className="font-playfair text-3xl md:text-4xl text-amber-200">
              Zarqaa al-Yamama
            </h2>
          </div>

          <div className="space-y-5 text-slate-300 text-lg leading-relaxed">
            <p>
              In ancient Arabia, a legendary seer named Zarqaa al-Yamama could spot armies
              hidden beyond the horizon — days before they arrived. Her eyes cut through dust,
              distance, and deception. No enemy could surprise her people while she watched.
            </p>
            <p>
              Web3 transactions work the same way an ambush does. A swap touches five contracts.
              One is a proxy. The implementation was upgraded three days ago. The audit is two years
              old. Nobody told you.{' '}
              <span className="text-amber-200 font-medium">Those are the hidden soldiers.</span>
            </p>
            <p>
              Zarqaa resolves the full execution path of every transaction — every proxy hop,
              every delegatecall, every bridge handoff — and returns a verdict for each leg before
              your wallet asks you to sign.
            </p>
          </div>

          <blockquote className="border-l-2 border-amber-500 pl-5 py-1">
            <p className="text-amber-400 text-xl font-playfair italic">
              "Silence is never treated as safe."
            </p>
            <p className="text-slate-500 text-sm mt-1">
              Every unresolvable component is flagged — nothing is silently assumed green.
            </p>
          </blockquote>
        </div>
      </section>

      {/* How it works */}
      <section className="border-t border-slate-800/60 py-20 px-4">
        <div className="max-w-4xl mx-auto space-y-12">

          <div className="text-center space-y-2">
            <p className="text-amber-500 text-sm tracking-widest uppercase">How it works</p>
            <h2 className="font-playfair text-3xl md:text-4xl text-amber-200">
              Eight stages. One verdict.
            </h2>
          </div>

          <div className="grid md:grid-cols-2 gap-4">
            {[
              { n: '01', title: 'Path resolution',    desc: 'Decode the full call graph — every contract leg, proxy hop, and delegatecall.' },
              { n: '02', title: 'Source verification', desc: 'Check that source code is verified on-chain and matches what you see.' },
              { n: '03', title: 'Ownership mapping',  desc: 'Identify who controls each contract — EOA, multisig, DAO, or unknown.' },
              { n: '04', title: 'Audit freshness',    desc: 'Compare last audit date against last upgrade — stale audits flag amber.' },
              { n: '05', title: 'Exploit scan',       desc: 'Cross-reference CVEs, Rekt database, and recent post-mortems.' },
              { n: '06', title: 'MEV analysis',       desc: 'Detect sandwich risk and recommend private RPC if exposure is high.' },
              { n: '07', title: 'Trust aggregation',  desc: 'Per-leg scores weighted by confidence — weakest leg drives route verdict.' },
              { n: '08', title: 'Streamed report',    desc: 'Results render as they arrive — Green / Amber / Red / Unverified.' },
            ].map(({ n, title, desc }) => (
              <div key={n} className="flex gap-4 p-5 rounded-xl border border-slate-800 bg-slate-900/40 hover:border-amber-500/30 transition-colors">
                <span className="text-amber-500/50 font-mono text-sm pt-1 shrink-0">{n}</span>
                <div>
                  <p className="text-amber-200 font-medium mb-1">{title}</p>
                  <p className="text-slate-400 text-sm leading-relaxed">{desc}</p>
                </div>
              </div>
            ))}
          </div>

          <div className="text-center">
            <Link href="/scanner">
              <Button
                size="lg"
                className="bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold text-lg px-10 py-6 rounded-xl shadow-[0_0_30px_rgba(245,158,11,0.25)] transition-all"
              >
                Try it now
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-slate-800/60 py-8 text-center text-slate-600 text-sm">
        Zarqaa — transaction-time security intelligence for Web3.
        Silence is never treated as safe.
      </footer>

    </main>
  )
}
