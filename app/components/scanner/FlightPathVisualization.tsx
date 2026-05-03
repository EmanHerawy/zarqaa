import { Leg } from '@/types/api'
import { Card } from '@/components/ui/card'

interface FlightPathVisualizationProps {
  legs: Leg[]
}

export default function FlightPathVisualization({ legs }: FlightPathVisualizationProps) {
  return (
    <Card className="border-slate-800 bg-slate-900/50 p-8">
      <div className="space-y-2">
        <h2 className="text-lg font-playfair text-amber-200 mb-4">Flight Path</h2>
        
        {/* SVG Timeline with Flight Path visualization */}
        <svg
          className="w-full h-auto min-h-48"
          viewBox={`0 0 120 ${legs.length * 25 + 30}`}
          preserveAspectRatio="xMidYMid meet"
        >
          {/* Render vertical path with nodes */}
          {legs.map((leg, index) => {
            const y = index * 25 + 20
            const nextY = (index + 1) * 25 + 20
            const getColor = (verdict: string) => {
              switch (verdict) {
                case 'green': return '#10b981'
                case 'amber': return '#f59e0b'
                case 'unverified': return '#6b7280'
                case 'red': return '#ef4444'
                default: return '#6b7280'
              }
            }
            const color = getColor(leg.verdict)

            return (
              <g key={index}>
                {/* Connecting Line to next leg (if not last) - with glow effect preparation */}
                {index < legs.length - 1 && (
                  <>
                    {/* Glow line for animated effect */}
                    <line
                      x1="60"
                      y1={y}
                      x2="60"
                      y2={nextY}
                      stroke={color}
                      strokeWidth="2"
                      opacity="0.2"
                    />
                    {/* Main line */}
                    <line
                      x1="60"
                      y1={y}
                      x2="60"
                      y2={nextY}
                      stroke={color}
                      strokeWidth="1"
                      opacity="0.7"
                    />
                  </>
                )}

                {/* Node Circle with outer ring */}
                <circle
                  cx="60"
                  cy={y}
                  r="6"
                  fill={color}
                  opacity="0.2"
                />
                <circle
                  cx="60"
                  cy={y}
                  r="3"
                  fill={color}
                />

                {/* Leg Index Label */}
                <text
                  x="20"
                  y={y + 1}
                  fontSize="2.5"
                  fill="#94a3b8"
                  textAnchor="middle"
                  fontWeight="bold"
                >
                  {index + 1}
                </text>
              </g>
            )
          })}
        </svg>

        {/* Legend */}
        <div className="flex flex-wrap gap-4 pt-4 mt-4 border-t border-slate-800">
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <div className="w-2 h-2 rounded-full bg-emerald-500" />
            <span>Verified Safe</span>
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <div className="w-2 h-2 rounded-full bg-amber-500" />
            <span>Review Required</span>
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <div className="w-2 h-2 rounded-full bg-slate-500" />
            <span>Unverified</span>
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <div className="w-2 h-2 rounded-full bg-rose-500" />
            <span>High Risk</span>
          </div>
        </div>
      </div>
    </Card>
  )
}
