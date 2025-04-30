import { Observable, observable } from '@legendapp/state'
import { For, use$ } from '@legendapp/state/react'
import type { LucideIcon } from 'lucide-react'

import { cn } from '#/lib/utils'

import { Button } from './ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from './ui/tooltip'

export interface TitleBarAction {
	tooltip: string
	Icon: LucideIcon
	iconClassName?: string
	onClick: () => void
}

export const titleBarActions$ = observable(new Map<string, Observable<TitleBarAction>>())

export default function TitleBar() {
	return (
		<div
			className="h-(--titlebar-height) border-b border-border w-full select-none flex items-center justify-between"
			data-tauri-drag-region
		>
			{/* NOTE: tauri overlays the titlebar with stoplight and title, but we can add buttons at the end  */}
			<div />
			<div className="flex items-center gap-1 mr-2.5">
				<For each={titleBarActions$}>
					{(action$) => {
						const { tooltip, Icon, iconClassName, onClick } = use$(action$)

						return (
							<Tooltip>
								<TooltipTrigger asChild>
									<Button onClick={onClick} size="icon-sm" variant="ghost">
										<Icon className={cn(iconClassName)} />
									</Button>
								</TooltipTrigger>
								<TooltipContent>{tooltip}</TooltipContent>
							</Tooltip>
						)
					}}
				</For>
			</div>
		</div>
	)
}
