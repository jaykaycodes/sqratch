import { reactive } from '@legendapp/state/react'
import type { LucideIcon } from 'lucide-react'

import { useIsProjectRoute } from '#/lib/hooks/use-project'
import { cn } from '#/lib/utils'
import ui$ from '#/stores/ui-store'

import Icons from './icons'
import Tooltip from './ui/tooltip'

export default function TitleBar() {
	const isProject = useIsProjectRoute()

	return (
		<div
			className="h-(--titlebar-height) border-b border-border w-full select-none flex items-center justify-between"
			data-tauri-drag-region
		>
			{/* NOTE: tauri overlays the titlebar with stoplight and title, but we can add buttons at the end  */}
			<div />
			<div className="flex items-center gap-1 mr-2.5">{isProject ? <ProjectActions /> : null}</div>
		</div>
	)
}

interface ActionButtonProps {
	label: string
	Icon: LucideIcon
	isActive?: boolean
	className?: string
	onClick?: () => void
}

const ActionButton = reactive(
	({ label, Icon, className, isActive, onClick }: ActionButtonProps) => (
		<Tooltip offsetBy={10} placement="bottom-start" tip={label}>
			<button
				className="btn btn-ghost btn-square btn-xs hover:shadow-none"
				onClick={onClick}
				type="button"
			>
				<Icon className={cn(isActive ? 'text-base-content' : 'text-base-content/80', className)} />
			</button>
		</Tooltip>
	),
)

function ProjectActions() {
	return (
		<>
			<ActionButton
				$isActive={ui$.detailsPanel.open}
				$label={() => (ui$.detailsPanel.open.get() ? 'Hide Details Panel' : 'Show Details Panel')}
				Icon={Icons.PanelRight}
				onClick={() => ui$.detailsPanel.open.toggle()}
			/>
			<ActionButton
				$isActive={ui$.detailsPanel.open}
				$label={() => (ui$.detailsPanel.open.get() ? 'Hide Details Panel' : 'Show Details Panel')}
				Icon={Icons.PanelRight}
				onClick={() => ui$.detailsPanel.open.toggle()}
			/>
			<ActionButton
				$isActive={ui$.detailsPanel.open}
				$label={() => (ui$.detailsPanel.open.get() ? 'Hide Details Panel' : 'Show Details Panel')}
				Icon={Icons.PanelRight}
				onClick={() => ui$.detailsPanel.open.toggle()}
			/>
		</>
	)
}
