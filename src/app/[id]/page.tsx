import Link from 'next/link'
import { notFound } from 'next/navigation'

// Sample diary entries data - you can replace this with your actual entries
const diaryEntries = {
  'thoughts': {
    title: 'Thoughts',
    content: `
      Today I decided to start this diary. It feels like the right time to document my thoughts and experiences.
      
      I've been thinking about how important it is to capture moments, both big and small. Life moves so quickly, and sometimes we forget the details that make each day unique.
      
      This morning, I woke up with a sense of clarity I haven't felt in a while. Maybe it's the new year, or maybe it's just time for a change. Whatever it is, I want to hold onto this feeling.
      
      I'm not sure what this diary will become, but I'm excited to find out. Maybe it will be a place for reflection, or maybe it will just be a record of my days. Either way, it feels right.
      
      Here's to new beginnings and the stories we'll tell.
    `
  }
}

interface PageProps {
  params: {
    id: string
  }
}

export default function DiaryEntry({ params }: PageProps) {
  const entry = diaryEntries[params.id as keyof typeof diaryEntries]

  if (!entry) {
    notFound()
  }

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-2xl mx-auto">
        <Link href="/" className="text-blue-600 hover:underline mb-8 inline-block">
          ← back
        </Link>
        
        <h1 className="text-2xl font-bold mb-6">{entry.title}</h1>
        
        <div className="prose">
          {entry.content.split('\n\n').map((paragraph, index) => (
            <p key={index} className="mb-4 leading-relaxed">
              {paragraph.trim()}
            </p>
          ))}
        </div>
      </div>
    </div>
  )
} 