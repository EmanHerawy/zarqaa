'use client'

import { motion } from 'framer-motion'
import LegCard from './LegCard'
import { Leg } from '@/types/api'

interface AnimatedLegCardProps {
  leg: Leg
  index: number
  totalLegs: number
}

export default function AnimatedLegCard({ leg, index, totalLegs }: AnimatedLegCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{
        duration: 0.5,
        delay: index * 0.1,
        ease: 'easeOut',
      }}
    >
      <LegCard leg={leg} index={index} totalLegs={totalLegs} />
    </motion.div>
  )
}
