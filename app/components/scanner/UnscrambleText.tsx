'use client'

import { useEffect, useState } from 'react'

interface UnscrambleTextProps {
  text: string
  className?: string
}

export default function UnscrambleText({ text, className = '' }: UnscrambleTextProps) {
  const [displayed, setDisplayed] = useState(false)

  useEffect(() => {
    // Trigger animation on mount
    setDisplayed(true)
  }, [])

  return (
    <div className={`text-unscramble ${className}`}>
      {text}
    </div>
  )
}
