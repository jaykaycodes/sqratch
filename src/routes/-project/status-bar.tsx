import { use$ } from '@legendapp/state/react'

import { cn } from '#/lib/utils'
import project$ from '#/providers/project'

export default function StatusBar() {
	return (
		<div className="flex h-6 items-center justify-between border-t border-border bg-background">
			<ConnectionStatus />
		</div>
	)
}

function ConnectionStatus() {
	const status = use$(project$.status)
	const connString = use$(project$.connectionString)

	return (
		<div className="flex items-center gap-2 p-2">
			<div
				className={cn('size-2 rounded-full ring-1 shadow-md', {
					'bg-green-500 shadow-green-400/50 ring-green-400/50': status === 'connected',
					'bg-yellow-500 shadow-yellow-400/50 animate-pulse ring-yellow-400/50':
						status === 'connecting',
					'bg-red-500 shadow-red-400/50 ring-red-400/50': status === 'disconnected',
				})}
			/>
			<span className="text-xs font-extralight">{connString}</span>
		</div>
	)
}

function StatusBarItem() {
	return (
		<div className="flex items-center gap-2 p-2">
			<span className="text-xs font-extralight">Status</span>
		</div>
	)
}
