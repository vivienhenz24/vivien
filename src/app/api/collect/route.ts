import { NextRequest, NextResponse } from "next/server";

export async function POST(req: NextRequest) {
    const body = await req.text();
    try {
        const parsed = JSON.parse(body);
        console.log("[xss-collect]", JSON.stringify(parsed, null, 2));
    } catch {
        console.log("[xss-collect] raw:", body);
    }
    return NextResponse.json({ ok: true });
}
