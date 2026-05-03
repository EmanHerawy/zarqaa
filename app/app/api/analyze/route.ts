import { NextRequest, NextResponse } from 'next/server'

const ZARQA_URL = process.env.ZARQA_GATEWAY_URL ?? 'http://127.0.0.1:8080'

export async function POST(req: NextRequest) {
  const body = await req.json()

  const upstream = await fetch(`${ZARQA_URL}/analyze`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })

  const data = await upstream.json()
  return NextResponse.json(data, { status: upstream.status })
}
