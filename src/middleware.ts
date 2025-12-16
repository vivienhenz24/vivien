import { fuzzyCanaryMiddleware } from '@fuzzycanary/core/middleware'
import { NextRequest, NextResponse } from 'next/server'

export async function middleware(request: NextRequest) {
  // For Next.js static sites, we need to get the static HTML response
  // and then let fuzzyCanaryMiddleware strip the canary for allowlisted bots
  const upstream = async () => {
    // Get the static HTML response
    return NextResponse.next()
  }

  return fuzzyCanaryMiddleware(request, upstream)
}

// Optional: Configure which routes the middleware runs on
export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    '/((?!api|_next/static|_next/image|favicon.ico).*)',
  ],
}

