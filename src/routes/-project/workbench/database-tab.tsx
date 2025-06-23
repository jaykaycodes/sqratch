import { type ItemInstance, selectionFeature, syncDataLoaderFeature } from '@headless-tree/core'
import { useTree } from '@headless-tree/react'
import { useSuspenseQuery } from '@tanstack/react-query'

import Icons from '#/components/icons'
import Q from '#/lib/queries'
import type { Entity } from '#/lib/taurpc'
import { cn } from '#/lib/utils'
import uiStore$ from '#/stores/ui-store'

const rootItemId = '__root'

export default function DatabaseTab() {
	const entitiesQuery = useSuspenseQuery(Q.db.entities)

	const tree = useTree<Entity>({
		rootItemId,
		getItemName: (item) => item.getItemData().name,
		isItemFolder: (item) => item.getItemData().kind === 'Schema',
		onPrimaryAction(item) {
			uiStore$.detailsPanel.activeObjectId.set(item.getItemData().id)
		},
		dataLoader: {
			getItem: (itemId) => {
				const item = entitiesQuery.data.find((e) => e.id === itemId)
				if (!item) throw new Error(`Item ${itemId} not found`)

				return item
			},
			getChildren: (itemId) => {
				const publicSchema = entitiesQuery.data.find((e) => e.name === 'public')
				if (itemId === rootItemId) {
					return Object.values(entitiesQuery.data)
						.filter(
							(e) =>
								((e.kind === 'Schema' && e.id !== publicSchema?.id) ||
									e.schemaId === publicSchema?.id) &&
								e.isSystem === false,
						)
						.map((e) => e.id)
						.sort((a, b) => {
							// Get the entities for comparison
							const entityA = entitiesQuery.data.find((e) => e.id === a)
							const entityB = entitiesQuery.data.find((e) => e.id === b)

							if (!entityA || !entityB) return 0

							// Sort schemas last
							if (entityA.kind === 'Schema' && entityB.kind !== 'Schema') return 1
							if (entityA.kind !== 'Schema' && entityB.kind === 'Schema') return -1

							// Otherwise sort alphabetically by name
							return entityA.name.localeCompare(entityB.name)
						})
				}

				return entitiesQuery.data.filter((e) => e.schemaId === itemId).map((e) => e.id)
			},
		},
		features: [syncDataLoaderFeature, selectionFeature],
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

function TreeRow({ item }: { item: ItemInstance<Entity> }) {
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
