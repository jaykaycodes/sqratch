import React from 'react'

import { type Observable } from '@legendapp/state'
import { Show, use$ } from '@legendapp/state/react'
import { $React } from '@legendapp/state/react-web'

import Icons from '#/components/icons'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { useProjectStore$ } from '#/stores/project-store'

import { itemIcons, type WorkbenchItem } from './misc'

interface WorkbenchItemProps {
	item$: Observable<WorkbenchItem>
}

export default function WorkbenchItemRow({ item$ }: WorkbenchItemProps) {
	const project$ = useProjectStore$()

	const Icon = use$(() =>
		React.cloneElement(itemIcons[item$.type.get()], {
			className: cn('text-muted-foreground size-3', itemIcons[item$.type.get()].props.className),
		}),
	)

	return (
		<Button
			asDiv
			className={'flex items-center w-full px-3 h-6 text-sm hover:bg-muted/50'}
			onClick={() => {
				// if (item$.type.get() === 'Folder') {
				// 	if (project$.ui.workbench.expanded.has(item$.id.get())) {
				// 		project$.ui.workbench.expanded.delete(item$.id.get())
				// 	} else {
				// 		project$.ui.workbench.expanded.add(item$.id.get())
				// 	}
				// }
			}}
			style={{
				marginLeft: item$.path.length ? item$.path.length * 8 : 0,
			}}
		>
			<Show if={item$.type.get() === 'Folder'}>
				<Icons.ChevronRight className="size-3 text-muted-foreground" />
			</Show>
			{Icon}
			<$React.span className="truncate">{item$.name.get()}</$React.span>
			{/* {item.hint && <span className="ml-1 text-xs text-muted-foreground">({item.hint})</span>}
			{onToggleFavorite && (
				<div className="ml-auto h-5 w-5 p-0 flex items-center justify-center hover:bg-muted rounded-sm">
					{item.favorited ? (
						<Icons.Star className="h-3 w-3 fill-yellow-400 text-yellow-400" />
					) : (
						<Icons.Star className="h-3 w-3 text-muted-foreground" />
					)}
				</div>
			)} */}
		</Button>
	)
}
