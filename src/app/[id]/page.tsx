import Link from 'next/link'
import { notFound } from 'next/navigation'
import { getContentById } from '@/lib/content'

export default async function ContentPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params
  const content = getContentById(id)

  if (!content) {
    notFound()
  }

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-3xl mx-auto">
        <Link href="/" className="text-blue-600 hover:underline mb-8 inline-block">
          ← back
        </Link>
        
        <article className="prose prose-lg max-w-none essay-content">
          <div className="mb-8">
            <div className="text-xl text-black font-normal mb-2">{content.title}</div>
            <time className="text-gray-600 text-sm">{content.date}</time>
          </div>
          
          <div 
            className="leading-relaxed"
            dangerouslySetInnerHTML={{ __html: content.htmlContent }}
          />
        </article>
      </div>
    </div>
  )
} 