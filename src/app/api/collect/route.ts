import { NextRequest, NextResponse } from "next/server";

export async function POST(req: NextRequest) {
    const body = await req.json();
    console.log("[xss-collect]", JSON.stringify(body, null, 2));
    return NextResponse.json({ ok: true });
}
