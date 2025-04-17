import React from 'react'

import { use$ } from '@legendapp/state/react'
import type { LucideIcon, LucideProps } from 'lucide-react'

import Icons from '#/components/icons'
import { Button } from '#/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '#/components/ui/tooltip'
import { TooltipProvider } from '#/components/ui/tooltip'
import { cn, copyToClipboard } from '#/lib/utils'
import { useProjectStore$ } from '#/providers/project'

export default function StatusBar() {
	return (
		<div className="flex h-6 items-center justify-between border-t border-border bg-background overflow-x-auto">
			<ConnectionStatusItem />
		</div>
	)
}

interface StatusBarItemProps {
	label: string
	icon?: LucideIcon
	iconProps?: LucideProps
	labelClassName?: string
	className?: string
	tooltip?: string
	onClick?: () => void
}

function StatusBarItem({
	label,
	icon,
	iconProps,
	className,
	tooltip,
	labelClassName,
	onClick,
}: StatusBarItemProps) {
	const Icon = React.useMemo(
		() =>
			icon
				? React.createElement(icon, {
						...iconProps,
						className: cn(iconProps?.className, 'size-2'),
					})
				: null,
		[icon, iconProps],
	)

	const body = (
		<Button
			asDiv
			className={cn('flex items-center gap-1', className)}
			onClick={onClick}
			variant="ghost"
		>
			{Icon}
			<span className={cn('text-2xs', labelClassName)}>{label}</span>
		</Button>
	)

	return tooltip ? (
		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger asChild>{body}</TooltipTrigger>
				<TooltipContent>{tooltip}</TooltipContent>
			</Tooltip>
		</TooltipProvider>
	) : (
		body
	)
}

function ConnectionStatusItem() {
	const store$ = useProjectStore$()
	const status = use$(store$.connectionStatus)
	const connStr = use$(store$.connectionString)

	let name = 'Untitled'
	try {
		const url = new URL(connStr)
		name = `${url.hostname}${url.port ? `:${url.port}` : ''}`
	} catch (error) {
		console.error(error)
	}

	return (
		<StatusBarItem
			className="cursor-copy"
			icon={Icons.Circle}
			iconProps={{
				fill: 'currentColor',
				className: cn('shadow-md rounded-full inset-ring-1', {
					'text-green-500 shadow-green-400/50 inset-ring-green-400/50': status === 'connected',
					'text-yellow-500 shadow-yellow-400/50 animate-pulse inset-ring-yellow-400/50':
						status === 'loading',
					'text-red-500 shadow-red-400/50 inset-ring-red-400/50': status === 'disconnected',
				}),
			}}
			label={name}
			onClick={() => copyToClipboard(connStr)}
		/>
	)
}
