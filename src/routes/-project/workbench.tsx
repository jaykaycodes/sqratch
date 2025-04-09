import React from 'react'

import { use$ } from '@legendapp/state/react'
import type { LucideIcon } from 'lucide-react'

import Icons from '#/components/icons'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '#/components/ui/collapsible'
import { ScrollArea } from '#/components/ui/scroll-area'
import { cn } from '#/lib/utils'
import project$ from '#/providers/project'

interface WorkbenchItem {
	id: string
	type: 'table' | 'function' | 'view' | 'query'
	name: string
	hint?: string
	favorited?: boolean
}

interface WorkbenchGroup {
	id: string
	type: 'schema' | 'folder'
	name: string
	items: (WorkbenchItem | WorkbenchGroup)[]
}

// Helper to get the icon for each item type
function getItemIcon(type: string, isOpen?: boolean): LucideIcon {
	switch (type) {
		case 'table':
			return Icons.Table
		case 'function':
			return Icons.Code
		case 'view':
			return Icons.LayoutGrid
		case 'query':
			return Icons.FileText
		case 'folder':
			return isOpen ? Icons.FolderOpen : Icons.Folder
		case 'schema':
			return Icons.Database
		default:
			return Icons.File
	}
}

interface WorkbenchItemProps {
	item: WorkbenchItem
	onToggleFavorite?: (id: string) => void
	onSelect?: (item: WorkbenchItem) => void
}

function WorkbenchItemRow({ item, onToggleFavorite, onSelect }: WorkbenchItemProps) {
	const Icon = getItemIcon(item.type)

	return (
		<div className="flex items-center w-full px-3 h-6 text-sm hover:bg-muted/50 rounded-sm">
			<Icon className="mr-1" size={12} />
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

interface WorkbenchGroupProps {
	group: WorkbenchGroup
	depth?: number
	onToggleFavorite?: (id: string) => void
	onSelect?: (item: WorkbenchItem) => void
}

function WorkbenchGroupRow({ group, depth = 0, onToggleFavorite, onSelect }: WorkbenchGroupProps) {
	const [isOpen, setIsOpen] = React.useState(false)

	return (
		<div className="space-y-0.5">
			<div
				className="flex items-center w-full px-1 h-6 text-sm hover:bg-muted/50 rounded-sm cursor-pointer"
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
								<WorkbenchGroupRow
									depth={depth + 1}
									group={item as WorkbenchGroup}
									onSelect={onSelect}
									onToggleFavorite={onToggleFavorite}
								/>
							) : (
								<WorkbenchItemRow
									item={item as WorkbenchItem}
									onSelect={onSelect}
									onToggleFavorite={onToggleFavorite}
								/>
							)}
						</React.Fragment>
					))}
				</div>
			)}
		</div>
	)
}

interface WorkbenchSectionProps {
	icon: LucideIcon
	title: string
	defaultOpen?: boolean
	className?: string
	children: React.ReactNode
}

function WorkbenchSection({
	icon: Icon,
	title,
	defaultOpen = true,
	className,
	children,
}: WorkbenchSectionProps) {
	return (
		<Collapsible className={cn('border-b', className)} defaultOpen={defaultOpen}>
			<CollapsibleTrigger asChild>
				<div className="flex h-6 w-full items-center justify-between px-2 py-0.5 hover:bg-muted/50 cursor-pointer">
					<div className="flex items-center gap-1">
						<Icon size={14} />
						<span className="font-medium text-xs">{title}</span>
					</div>
					<Icons.ChevronDown className="text-muted-foreground" size={14} />
				</div>
			</CollapsibleTrigger>
			<CollapsibleContent>
				<ScrollArea className="px-1 py-1" style={{ height: 'var(--section-height, 180px)' }}>
					<div className="space-y-0.5">{children}</div>
				</ScrollArea>
			</CollapsibleContent>
		</Collapsible>
	)
}

// Example data - in a real app this would come from API/context
const EXAMPLE_FAVORITES: WorkbenchItem[] = [
	{ id: 'fav1', type: 'query', name: 'Monthly active users', favorited: true },
	{ id: 'fav2', type: 'table', name: 'users', favorited: true },
	{ id: 'fav3', type: 'query', name: 'Revenue report', favorited: true },
]

const EXAMPLE_QUERIES: WorkbenchGroup[] = [
	{
		id: 'dir1',
		type: 'folder',
		name: 'Reports',
		items: [
			{ id: 'q1', type: 'query', name: 'Monthly active users', favorited: true },
			{ id: 'q2', type: 'query', name: 'Revenue report', favorited: true },
			{
				id: 'subdir1',
				type: 'folder',
				name: 'Marketing',
				items: [{ id: 'q3', type: 'query', name: 'Campaign performance', favorited: false }],
			},
		],
	},
	{
		id: 'dir2',
		type: 'folder',
		name: 'Analytics',
		items: [
			{ id: 'q4', type: 'query', name: 'User retention', favorited: false },
			{ id: 'q5', type: 'query', name: 'Funnel analysis', favorited: false },
		],
	},
]

const EXAMPLE_ENTITIES: WorkbenchGroup[] = [
	{
		id: 'schema1',
		type: 'schema',
		name: 'public',
		items: [
			{ id: 'table1', type: 'table', name: 'users', hint: '15 cols', favorited: true },
			{ id: 'table2', type: 'table', name: 'products', hint: '8 cols', favorited: false },
			{ id: 'view1', type: 'view', name: 'active_users', favorited: false },
			{ id: 'func1', type: 'function', name: 'get_user_by_email', favorited: false },
		],
	},
	{
		id: 'schema2',
		type: 'schema',
		name: 'auth',
		items: [
			{ id: 'table3', type: 'table', name: 'users', hint: '10 cols', favorited: false },
			{ id: 'table4', type: 'table', name: 'sessions', hint: '5 cols', favorited: false },
		],
	},
]

export default function ProjectWorkbench() {
	const schema = use$(project$.entities)

	console.log(schema)

	// In a real implementation, these would be state variables with proper handlers
	const [favorites, setFavorites] = React.useState(EXAMPLE_FAVORITES)
	const [queries, setQueries] = React.useState(EXAMPLE_QUERIES)
	const [entities, setEntities] = React.useState(EXAMPLE_ENTITIES)

	const handleToggleFavorite = React.useCallback((id: string) => {
		// This is just a stub - in a real app, this would update the server
		console.log('Toggle favorite for:', id)
	}, [])

	const handleSelectItem = React.useCallback((item: WorkbenchItem) => {
		// This is just a stub - in a real app, this would open the item
		console.log('Selected item:', item)
	}, [])

	return (
		<div className="flex grow flex-col">
			{/* Favorites section */}
			<WorkbenchSection
				className="var(--section-height, 120px)"
				defaultOpen
				icon={Icons.Star}
				title="Favorites"
			>
				{favorites.length === 0 ? (
					<div className="flex items-center justify-center h-16 text-muted-foreground">
						<span className="text-xs">No favorites yet</span>
					</div>
				) : (
					favorites.map((item) => (
						<WorkbenchItemRow
							item={item}
							key={item.id}
							onSelect={handleSelectItem}
							onToggleFavorite={handleToggleFavorite}
						/>
					))
				)}
			</WorkbenchSection>

			{/* Entities section */}
			<WorkbenchSection className="flex-1" defaultOpen icon={Icons.Database} title="Entities">
				{entities.length === 0 ? (
					<div className="flex items-center justify-center h-16 text-muted-foreground">
						<span className="text-xs">No database entities found</span>
					</div>
				) : (
					entities.map((schema) => (
						<WorkbenchGroupRow
							group={schema}
							key={schema.id}
							onSelect={handleSelectItem}
							onToggleFavorite={handleToggleFavorite}
						/>
					))
				)}
			</WorkbenchSection>

			{/* Queries section */}
			<WorkbenchSection
				className="var(--section-height, 200px)"
				defaultOpen
				icon={Icons.FileCode}
				title="Queries"
			>
				{queries.length === 0 ? (
					<div className="flex items-center justify-center h-16 text-muted-foreground">
						<span className="text-xs">No queries found</span>
					</div>
				) : (
					queries.map((group) => (
						<WorkbenchGroupRow
							group={group}
							key={group.id}
							onSelect={handleSelectItem}
							onToggleFavorite={handleToggleFavorite}
						/>
					))
				)}
			</WorkbenchSection>
		</div>
	)
}
