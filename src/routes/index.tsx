import type React from 'react'
import { useEffect, useState } from 'react'

import { basename } from '@tauri-apps/api/path'
import { getMatches } from '@tauri-apps/plugin-cli'
import { readTextFile } from '@tauri-apps/plugin-fs'

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

// Define types for connection info
interface ConnectionInfo {
	name: string
	connectionString: string
	timestamp: number
	path: string
}

function HomePage() {
	const [connections, setConnections] = useAtom(connectionsAtom)
	const [isAddingNew, setIsAddingNew] = useState(false)
	const [connectionString, setConnectionString] = useState('')
	const [isLoading, setIsLoading] = useState(true)

	// Check for project connection on mount
	useEffect(() => {
		const loadProjectConnection = async () => {
			try {
				setIsLoading(true)

				// Get the project path from CLI arguments
				const matches = await getMatches()
				const projectPath = matches.args['project-path']?.value as string

				if (!projectPath) {
					console.log('No project path provided')
					setIsLoading(false)
					return
				}

				// Try to load the current connection from the project
				try {
					const connectionPath = `${projectPath}/.sqratch/connections/current.json`
					const connectionData = await readTextFile(connectionPath)

					const connectionInfo = JSON.parse(connectionData) as ConnectionInfo

					// Only use connections that are less than 1 day old
					const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000
					if (connectionInfo.timestamp && connectionInfo.timestamp > oneDayAgo) {
						// Get the project name
						let projectName = connectionInfo.name

						if (!projectName) {
							try {
								// Try to get the project name from the config
								const configPath = `${projectPath}/.sqratch/config.json`
								const configData = await readTextFile(configPath)

								const config = JSON.parse(configData)
								projectName = config.settings.projectName

								if (!projectName) {
									// Fall back to directory name
									projectName = await basename(projectPath)
								}
							} catch (error) {
								// Couldn't read project config, use default name
								projectName = await basename(projectPath)
							}
						}

						handleAddConnection(connectionInfo.connectionString, projectName)
						toast.success(`Connected to ${projectName}`)
					}
				} catch (error) {
					console.error('Error loading project connection:', error)
				}
			} catch (error) {
				console.error('Error checking for project connection:', error)
			} finally {
				setIsLoading(false)
			}
		}

		loadProjectConnection()
	}, [])

	const handleAddConnection = (connString = connectionString, customName = '') => {
		try {
			const parsed = parseConnectionString(connString)
			const newConnection: DbConnection = {
				connectionString: connString,
				createdAt: Date.now(),
				id: nanoid(),
				name: customName || `Database at ${parsed.host}`,
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

	// Show loading state
	if (isLoading) {
		return (
			<div className="container mx-auto flex h-[80vh] items-center justify-center p-8">
				<div className="space-y-4 text-center">
					<Icons.Loader className="mx-auto h-12 w-12 animate-spin text-primary" />
					<p>Loading project connection...</p>
				</div>
			</div>
		)
	}

	// Show simplified view when no connections exist
	if (connections.length === 0) {
		return (
			<div className="container mx-auto p-8">
				<div className="mx-auto max-w-2xl space-y-4 text-center">
					<Icons.Database className="mx-auto h-12 w-12 text-muted-foreground" />
					<h1 className="font-bold text-2xl">Connect to Your Database</h1>
					<p className="text-muted-foreground">
						Paste your PostgreSQL connection string to get started
					</p>
					<div className="flex gap-2">
						<Input
							className="flex-1"
							onChange={(e) => setConnectionString(e.target.value)}
							onKeyDown={handleKeyDown}
							placeholder="postgresql://user:password@localhost:5432/dbname"
							value={connectionString}
						/>
						<Button onClick={() => handleAddConnection()}>
							<Icons.ArrowRight className="mr-2 h-4 w-4" />
							Connect
						</Button>
					</div>
				</div>
			</div>
		)
	}

	return (
		<div className="container mx-auto p-8">
			<div className="mb-8 space-y-4">
				<div className="flex items-center justify-between">
					<h1 className="font-bold text-2xl">Database Connections</h1>
					<Button onClick={() => setIsAddingNew(!isAddingNew)}>
						<Icons.Plus className="mr-2 h-4 w-4" />
						Add Connection
					</Button>
				</div>

				{isAddingNew && (
					<div className="flex gap-2">
						<Input
							autoFocus
							className="flex-1"
							onChange={(e) => setConnectionString(e.target.value)}
							onKeyDown={handleKeyDown}
							placeholder="postgresql://user:password@localhost:5432/dbname"
							value={connectionString}
						/>
						<Button onClick={() => handleAddConnection()}>
							<Icons.ArrowRight className="mr-2 h-4 w-4" />
							Connect
						</Button>
					</div>
				)}
			</div>

			<div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
				{connections.map((connection) => (
					<div
						className="rounded-lg border p-4 transition-colors hover:border-primary"
						key={connection.id}
					>
						<h3 className="mb-2 font-semibold">{connection.name}</h3>
						<div className="space-y-1 text-muted-foreground text-sm">
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
