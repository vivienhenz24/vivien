import Link from 'next/link'
import { notFound } from 'next/navigation'
import { getEssayById } from '@/lib/essays'

interface PageProps {
  params: {
    id: string
  }
}

export default function EssayPage({ params }: PageProps) {
  const essay = getEssayById(params.id)

  if (!essay) {
    notFound()
  }

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-3xl mx-auto">
        <Link href="/" className="text-blue-600 hover:underline mb-8 inline-block">
          ← back to essays
        </Link>
        
        <article className="prose prose-lg max-w-none">
          <div className="mb-8">
            <div className="text-2xl text-black font-semibold mb-2">{essay.title}</div>
            <time className="text-gray-600 text-sm">{essay.date}</time>
          </div>
          
          <div 
            className="leading-relaxed"
            dangerouslySetInnerHTML={{ __html: essay.htmlContent }}
          />
        </article>
      </div>
    </div>
  )
} 