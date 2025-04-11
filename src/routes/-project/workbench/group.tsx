import React from 'react'

import Icons from '#/components/icons'
import { cn } from '#/lib/utils'

import WorkbenchItemRow from './item'
import type { WorkbenchGroup } from './types'

interface WorkbenchGroupProps {
	group: WorkbenchGroup
	depth?: number
}

export default function WorkbenchGroupRow({ group, depth = 0 }: WorkbenchGroupProps) {
	const [isOpen, setIsOpen] = React.useState(false)

	return (
		<div className="space-y-0.5">
			<div
				className="flex items-center w-full px-1 h-6 text-sm hover:bg-muted/50 rounded-sm"
				onClick={() => setIsOpen(!isOpen)}
			>
				{group.items.length > 0 ? (
					<Icons.ChevronRight
						className={cn('h-3 w-3 mr-1 text-muted-foreground transition-transform', {
							'rotate-90': isOpen,
						})}
					/>
				) : (
					<div className="w-3 mr-1" />
				)}
				<span className="text-xs uppercase">{group.name}</span>
			</div>

			{isOpen && group.items.length > 0 && (
				<div className={cn('space-y-0.5', depth > 0 && 'ml-3')}>
					{group.items.map((item) => (
						<React.Fragment key={item.id}>
							{'items' in item ? (
								<WorkbenchGroupRow depth={depth + 1} group={item} />
							) : (
								<WorkbenchItemRow item={item} />
							)}
						</React.Fragment>
					))}
				</div>
			)}
		</div>
	)
}
