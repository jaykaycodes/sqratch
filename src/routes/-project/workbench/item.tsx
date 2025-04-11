import type { LucideIcon } from 'lucide-react'

import Icons from '#/components/icons'
import { cn } from '#/lib/utils'

import type { WorkbenchItem, WorkbenchItemType } from './types'

export const itemIcons = {
	Table: Icons.Table,
	ForeignTable: Icons.Table,
	View: Icons.LayoutGrid,
	MaterializedView: Icons.LayoutGrid,
	Function: Icons.Code,
	Procedure: Icons.Code,
	Sequence: Icons.Code,
	Type: Icons.Code,
	UserQuery: Icons.FileCode,
} satisfies Record<WorkbenchItemType, LucideIcon>

// Helper to get the icon for each item type
function getItemIcon(type: WorkbenchItemType): React.ReactNode {
	let Icon: LucideIcon
	const classNames = ['mr-1']

	switch (type) {
		case 'Table':
			Icon = Icons.Table
			break
		case 'ForeignTable':
			Icon = Icons.Table
			classNames.push('text-blue-500')
			break
		case 'View':
			Icon = Icons.View
			classNames.push('text-green-500')
			break
		case 'UserQuery':
			Icon = Icons.FileCode
			break
		case 'Function':
			Icon = Icons.FunctionSquare
			break
		case 'MaterializedView':
			Icon = Icons.View
			classNames.push('text-green-500')
			break
		case 'Procedure':
			Icon = Icons.FunctionSquare
			classNames.push('text-purple-500')
			break
		case 'Sequence':
			Icon = Icons.ArrowUp10
			classNames.push('text-blue-500')
			break
		case 'Type':
			Icon = Icons.Variable
			classNames.push('text-blue-500')
			break
		default:
			Icon = Icons.File
	}

	return <Icon className={cn(...classNames)} size={12} />
}

interface WorkbenchItemProps {
	item: WorkbenchItem
	onToggleFavorite?: (id: string) => void
}

export default function WorkbenchItemRow({ item, onToggleFavorite }: WorkbenchItemProps) {
	const Icon = getItemIcon(item.type)

	return (
		<div className="flex items-center w-full px-3 h-6 text-sm hover:bg-muted/50 rounded-sm">
			{Icon}
			<span className="text-xs">{item.name}</span>
			{item.hint && <span className="ml-1 text-xs text-muted-foreground">({item.hint})</span>}
			{onToggleFavorite && (
				<div className="ml-auto h-5 w-5 p-0 flex items-center justify-center hover:bg-muted rounded-sm">
					{item.favorited ? (
						<Icons.Star className="h-3 w-3 fill-yellow-400 text-yellow-400" />
					) : (
						<Icons.Star className="h-3 w-3 text-muted-foreground" />
					)}
				</div>
			)}
		</div>
	)
}
