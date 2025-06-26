import { useQuery, useSuspenseQuery } from '@tanstack/react-query'

import DivButton from '#/components/div-button'
import Tooltip from '#/components/ui/tooltip'
import Q from '#/lib/queries'
import { cn, copyToClipboard } from '#/lib/utils'

export default function StatusBar() {
	return (
		<div className="flex h-6 items-center justify-between border-t border-border bg-background overflow-x-auto overflow-y-hidden">
			<ConnectionStatusItem />
		</div>
	)
}

function ConnectionStatusItem() {
	const { data: project } = useSuspenseQuery(Q.project.get)

	const { data: isConnected, isLoading } = useQuery({
		...Q.db.isConnected,
		refetchOnWindowFocus: 'always',
		refetchInterval: ({ state }) => (state.data ? 10_000 : false),
	})

	return (
		<Tooltip placement="top-end" tip={project.db_url}>
			<DivButton
				className="cursor-copy btn-ghost bg-base-300 border-none shadow-none hover:bg-base-200"
				onClick={() => copyToClipboard(project.db_url)}
			>
				<div
					className={cn(
						'status',
						isLoading
							? 'animate-pulse status-warning'
							: isConnected
								? 'status-success'
								: 'status-error',
					)}
				/>
				<span className="text-2xs">{project.name}</span>
			</DivButton>
		</Tooltip>
	)
}
