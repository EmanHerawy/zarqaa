'use client'

import { motion } from 'framer-motion'
import VerdictBanner from './VerdictBanner'
import { VerdictStatus } from '@/types/api'

interface AnimatedVerdictBannerProps {
  verdict: VerdictStatus
  txHash: string | null
}

export default function AnimatedVerdictBanner({ verdict, txHash }: AnimatedVerdictBannerProps) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{
        duration: 0.4,
        ease: 'easeOut',
      }}
    >
      <VerdictBanner verdict={verdict} txHash={txHash} />
    </motion.div>
  )
}
