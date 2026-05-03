'use client'

import { useState } from 'react'
import Link from 'next/link'
import Image from 'next/image'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ArrowLeft, Zap } from 'lucide-react'
import ScannerResults from '@/components/scanner/ScannerResults'
import WaitingForSignal from '@/components/scanner/WaitingForSignal'
import { AnalysisResponse, AnalysisMode } from '@/types/api'

export default function ScannerPage() {
  const [mode, setMode] = useState<AnalysisMode>('tx_hash')
  const [input, setInput] = useState('')
  const [chain, setChain] = useState('ethereum')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [results, setResults] = useState<AnalysisResponse | null>(null)

  const handleAnalyze = async () => {
    setError(null)
    setLoading(true)
    setResults(null)

    try {
      const endpoint = mode === 'tx_hash' ? '/api/analyze' : '/api/analyze-intent'
      const payload = mode === 'tx_hash' 
        ? { tx_hash: input, chain }
        : { intent: input }

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`)
      }

      const data: AnalysisResponse = await response.json()
      setResults(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Analysis failed. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  const isValidInput = mode === 'tx_hash' 
    ? input.trim().length > 0
    : input.trim().length > 0

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100">
      {/* Header */}
      <header className="border-b border-slate-800 bg-slate-950/80 backdrop-blur-sm sticky top-0 z-50">
        <div className="max-w-6xl mx-auto px-4 py-4 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-2 hover:opacity-80 transition-opacity">
            <ArrowLeft className="w-5 h-5 text-amber-500" />
            <span className="text-amber-200 font-playfair text-xl">ZARQAA</span>
          </Link>
          <p className="text-slate-400 text-sm">The Scanner</p>
        </div>
      </header>

      {/* Main Content */}
      <div className={`max-w-6xl mx-auto px-4 py-8 ${loading ? 'scanning' : ''}`}>
        {!results ? (
          <div className="space-y-8">
            {/* Input Card */}
            <Card className="border-slate-800 bg-slate-900/50 backdrop-blur-sm p-8">
              <div className="space-y-6">
                {/* Mode Toggle */}
                <Tabs value={mode} onValueChange={(v) => setMode(v as AnalysisMode)}>
                  <TabsList className="grid w-full max-w-md grid-cols-2 bg-slate-800">
                    <TabsTrigger value="tx_hash" className="data-[state=active]:bg-amber-500 data-[state=active]:text-slate-950">
                      Transaction Hash
                    </TabsTrigger>
                    <TabsTrigger value="intent" className="data-[state=active]:bg-amber-500 data-[state=active]:text-slate-950">
                      Intent (Pre-sign)
                    </TabsTrigger>
                  </TabsList>

                  {/* Transaction Hash Tab */}
                  <TabsContent value="tx_hash" className="mt-6 space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-amber-200 mb-2">
                        Transaction Hash
                      </label>
                      <input
                        type="text"
                        placeholder="0xf1c5547d616f2d0ca31eae0c7e137109ce45da8bb8b91862b6c0f92711a5e1b5"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg text-slate-100 placeholder-slate-500 focus:outline-none focus:border-amber-500 focus:ring-1 focus:ring-amber-500"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-amber-200 mb-2">
                        Chain
                      </label>
                      <select
                        value={chain}
                        onChange={(e) => setChain(e.target.value)}
                        className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg text-slate-100 focus:outline-none focus:border-amber-500 focus:ring-1 focus:ring-amber-500"
                      >
                        <option value="ethereum">Ethereum</option>
                        <option value="polygon">Polygon</option>
                        <option value="arbitrum">Arbitrum</option>
                        <option value="optimism">Optimism</option>
                      </select>
                    </div>
                  </TabsContent>

                  {/* Intent Tab */}
                  <TabsContent value="intent" className="mt-6 space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-amber-200 mb-2">
                        Transaction Intent
                      </label>
                      <textarea
                        placeholder="I want to swap 1 ETH for USDC on Uniswap V3 on Ethereum"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg text-slate-100 placeholder-slate-500 focus:outline-none focus:border-amber-500 focus:ring-1 focus:ring-amber-500 resize-none h-24"
                      />
                    </div>
                  </TabsContent>
                </Tabs>

                {/* Error Message */}
                {error && (
                  <div className="p-4 bg-rose-900/30 border border-rose-700 rounded-lg text-rose-200 text-sm">
                    {error}
                  </div>
                )}

                {/* Analyze Button */}
                <Button
                  onClick={handleAnalyze}
                  disabled={!isValidInput || loading}
                  className="w-full bg-amber-500 hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed text-slate-950 font-bold py-3 rounded-lg transition-all duration-300 flex items-center justify-center gap-2"
                >
                  {loading ? (
                    <>
                      <div className="w-4 h-4 border-2 border-slate-950 border-t-transparent rounded-full animate-spin" />
                      Analyzing...
                    </>
                  ) : (
                    <>
                      <Zap className="w-5 h-5" />
                      Analyze Now
                    </>
                  )}
                </Button>
              </div>
            </Card>

            {/* Zero-State Placeholder */}
            <WaitingForSignal />
          </div>
        ) : (
          <ScannerResults results={results} onBack={() => setResults(null)} />
        )}
      </div>
    </main>
  )
}
