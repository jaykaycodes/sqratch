import {
	type ItemInstance,
	propMemoizationFeature,
	selectionFeature,
	syncDataLoaderFeature,
} from '@headless-tree/core'
import { useTree } from '@headless-tree/react'
import { use$ } from '@legendapp/state/react'
import { useSuspenseQuery } from '@tanstack/react-query'

import Icons from '#/components/icons'
import { useDoubleClick } from '#/lib/hooks/use-double-click'
import Q from '#/lib/queries'
import type { DbEntity } from '#/lib/taurpc'
import { cn } from '#/lib/utils'
import EditorStore$ from '#/stores/editor-store'
import WorkbenchStore$ from '#/stores/workbench-store'

const rootItemId = '__root' as const

export default function DatabaseTab() {
	const { data: entities } = useSuspenseQuery(Q.db.entities)
	console.log('entities', entities, Object.keys(entities).length)
	const treeState$ = WorkbenchStore$.treeStates.database
	const state = use$(treeState$)

	const { handleClick: onPrimaryAction } = useDoubleClick(
		() => {},
		// Handle double click: could expand/collapse folders or open in editor
		(item: ItemInstance<TreeItem>) => {
			const entity = item.getItemData()
			EditorStore$.openTab('entity', entity.id, entity.name)
		},
		{
			delay: 250,
		},
	)

	const tree = useTree<DbEntity>({
		state,
		rootItemId,
		features: [syncDataLoaderFeature, selectionFeature, propMemoizationFeature],
		onPrimaryAction,
		getItemName: (item) => item.getItemData().name,
		isItemFolder: (item) => 'children' in item.getItemData(),
		setExpandedItems: (u) =>
			typeof u === 'function'
				? treeState$.expandedItems.set(u(state.expandedItems ?? []))
				: treeState$.expandedItems.set(u),
		setFocusedItem: (u) =>
			typeof u === 'function'
				? treeState$.focusedItem.set(u(state.focusedItem ?? null))
				: treeState$.focusedItem.set(u),
		setSelectedItems: (u) =>
			typeof u === 'function'
				? treeState$.selectedItems.set(u(state.selectedItems ?? []))
				: treeState$.selectedItems.set(u),
		dataLoader: {
			getItem: (itemId) => {
				const item = entities[itemId]
				if (!item) throw new Error(`Item ${itemId} not found`)

				return item
			},
			getChildren: (itemId) => {
				if (itemId === rootItemId) return getRootChildren(entities as Record<string, DbEntity>)

				const entity = entities[itemId] ?? { children: [] }
				return 'children' in entity ? entity.children : []
			},
		},
	})

	return (
		<div {...tree.getContainerProps()} className="overflow-y-auto h-full rounded-md">
			<ul className="menu menu-md [&_li>*]:rounded-none w-full gap-0 p-0">
				<li className="menu-title">Database</li>
				{tree.getItems().map((item) => (
					<TreeRow item={item} key={item.getId()} />
				))}
			</ul>
		</div>
	)
}

function TreeRow({ item }: { item: ItemInstance<DbEntity> }) {
	const level = item.getItemMeta().level

	return (
		<li className="relative flex" key={item.getId()}>
			{level > 0 &&
				[...Array(level)].map((_, i) => (
					<span
						className="absolute top-0 bottom-0 w-px p-0 bg-base-content/20"
						key={`${item.getId()}-${i}`}
						style={{ left: `${i * 16 + 14}px` }}
					/>
				))}

			<button
				{...item.getProps()}
				className={cn(
					'flex cursor-default items-center gap-1 rounded-none h-full text-left py-1 hover:bg-base-200',
					{
						'menu-focused': item.isFocused(),
						'menu-active': item.isSelected(),
						// 'font-medium': item.isFolder(),
					},
				)}
				style={{
					paddingLeft: `${level * 16 + 14}px`,
				}}
			>
				{item.isFolder() && (
					<span
						className={cn('text-xs transition-transform duration-100', {
							'rotate-90': item.isExpanded(),
						})}
					>
						<Icons.ChevronRight className="size-4" />
					</span>
				)}
				{item.getItemName()}
			</button>
		</li>
	)
}

type TreeItem = DbEntity & {
	children?: string[]
}

/**
 * Build the root level of the tree
 */
function getRootChildren(entities: Record<string, DbEntity>): string[] {
	const schemas = Object.values(entities).filter((e) => e.kind === 'Schema')
	let publicTables: DbEntity[] = []

	const publicSchemaIndex = schemas.findIndex((e) => e.name === 'public')
	console.log('publicSchemaIndex', publicSchemaIndex)
	if (publicSchemaIndex !== -1) {
		const publicSchema = schemas.splice(publicSchemaIndex, 1)[0]
		publicTables = Object.values(entities).filter(
			(e) => e && 'schemaId' in e && e.schemaId === publicSchema.id,
		) as DbEntity[]
	}

	return [...publicTables, ...schemas]
		.sort((a, b) => {
			// Sort schemas last
			if (a.kind === 'Schema' && b.kind !== 'Schema') return 1
			if (a.kind !== 'Schema' && b.kind === 'Schema') return -1

			// Otherwise sort alphabetically by name
			return a.name.localeCompare(b.name)
		})
		.map((e) => e.id)
}
