import { NextResponse } from 'next/server'
import { getCanaryPayload, getCanaryHeader } from '@fuzzycanary/core'

export function middleware() {
  const payload = getCanaryPayload()
  const { name, value } = getCanaryHeader(payload)
  const res = NextResponse.next()
  res.headers.set(name, value)
  return res
}

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico).*)'],
}

