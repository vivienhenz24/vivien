import Link from 'next/link'

// Sample diary entries - you can replace these with your actual entries
const diaryEntries = [
  {
    id: 'thoughts',
    title: 'Thoughts'
  }
]

export default function Home() {
  return (
    <div className="min-h-screen p-8">
      <div className="max-w-md mx-auto">
        <h1 className="text-3xl font-bold mb-8">Vivien's Diary</h1>
        
        <div className="space-y-4">
          {diaryEntries.map((entry) => (
            <div key={entry.id}>
              <Link href={`/${entry.id}`} className="text-blue-600 hover:underline">
                {entry.title}
              </Link>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
