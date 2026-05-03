export type AnalysisMode = 'tx_hash' | 'intent'

export type VerdictStatus = 'green' | 'amber' | 'unverified' | 'red'

export interface BridgeInfo {
  protocol: string
  summary: string
  dvn_count: number
  controller_type: string
  upgrade_timelock_days: number | null
  relayer_type: string
  past_exploits_usd: number
  past_exploit_note: string
  last_audit: string
  bug_bounty_usd: number
  has_rate_limits: boolean
  has_circuit_breaker: boolean
  emergency_pause_by: string
  recent_flags: string[]
  recent_flags_source: {
    type: string
  }
  verdict_label: string
  verdict_summary: string
  centralization_risk: string
  destination_chain: string | null
  static_data_source: {
    type: string
  }
}

export interface Leg {
  address: string
  chain: string
  verdict: VerdictStatus
  source_verified: boolean
  is_proxy: boolean
  proxy_implementation: string | null
  infra_kind: string | null
  bridge_info: BridgeInfo | null
  notes: string[]
}

export interface MEVRisk {
  risk_level: 'low' | 'medium' | 'high'
  summary: string
  recommendation: string
  signals: string[]
}

export interface IntentResolution {
  normalized_to: string
  decoded_call: string
  unresolved_legs: string[]
}

export interface AnalysisResponse {
  tx_hash: string | null
  chain: string
  route_verdict: VerdictStatus
  legs: Leg[]
  mev_risk?: MEVRisk
  intent_resolution?: IntentResolution
}
