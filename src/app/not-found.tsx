import Link from 'next/link'

export default function NotFound() {
  return (
    <div className="min-h-screen flex items-center justify-center p-8">
      <div className="text-center">
        <h1 className="text-4xl font-normal mb-2">404</h1>
        <h2 className="text-lg font-normal mb-6 text-gray-700">Not Found</h2>
        

        
        <Link 
          href="/" 
          className="text-blue-600 underline hover:text-blue-800"
        >
          ← Home
        </Link>
      </div>
    </div>
  )
}
