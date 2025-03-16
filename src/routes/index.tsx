import { useState } from 'react'

import { createFileRoute } from '@tanstack/react-router'
import { useAtom } from 'jotai'
import * as Icons from 'lucide-react'
import { nanoid } from 'nanoid'
import { toast } from 'sonner'

import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { parseConnectionString } from '#/lib/utils'
import { connectionsAtom, type DbConnection } from '#/stores/projects-store'

export const Route = createFileRoute('/')({
  component: HomePage,
})

function HomePage() {
  const [connections, setConnections] = useAtom(connectionsAtom)
  const [isAddingNew, setIsAddingNew] = useState(false)
  const [connectionString, setConnectionString] = useState('')

  const handleAddConnection = (connString = connectionString) => {
    try {
      const parsed = parseConnectionString(connString)
      const newConnection: DbConnection = {
        id: nanoid(),
        name: `Database at ${parsed.host}`,
        connectionString: connString,
        createdAt: Date.now(),
        ...parsed,
      }

      setConnections((prev) => [...prev, newConnection])
      setConnectionString('')
      setIsAddingNew(false)
    } catch (error) {
      toast.error('Invalid connection string')
      console.error(error)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && connectionString) {
      handleAddConnection()
    } else if (e.key === 'Escape') {
      setIsAddingNew(false)
      setConnectionString('')
    }
  }

  // Show simplified view when no connections exist
  if (connections.length === 0) {
    return (
      <div className="container mx-auto p-8">
        <div className="max-w-2xl mx-auto text-center space-y-4">
          <Icons.Database className="w-12 h-12 mx-auto text-muted-foreground" />
          <h1 className="text-2xl font-bold">Connect to Your Database</h1>
          <p className="text-muted-foreground">
            Paste your PostgreSQL connection string to get started
          </p>
          <div className="flex gap-2">
            <Input
              value={connectionString}
              onChange={(e) => setConnectionString(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="postgresql://user:password@localhost:5432/dbname"
              className="flex-1"
            />
            <Button onClick={() => handleAddConnection()}>
              <Icons.ArrowRight className="w-4 h-4 mr-2" />
              Connect
            </Button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="container mx-auto p-8">
      <div className="space-y-4 mb-8">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold">Database Connections</h1>
          <Button onClick={() => setIsAddingNew(!isAddingNew)}>
            <Icons.Plus className="w-4 h-4 mr-2" />
            Add Connection
          </Button>
        </div>

        {isAddingNew && (
          <div className="flex gap-2">
            <Input
              value={connectionString}
              onChange={(e) => setConnectionString(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="postgresql://user:password@localhost:5432/dbname"
              className="flex-1"
              autoFocus
            />
            <Button onClick={() => handleAddConnection()}>
              <Icons.ArrowRight className="w-4 h-4 mr-2" />
              Connect
            </Button>
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {connections.map((connection) => (
          <div
            key={connection.id}
            className="p-4 border rounded-lg hover:border-primary transition-colors"
          >
            <h3 className="font-semibold mb-2">{connection.name}</h3>
            <div className="text-sm text-muted-foreground space-y-1">
              <p>Host: {connection.host}</p>
              <p>Database: {connection.database}</p>
              <p>User: {connection.user}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
