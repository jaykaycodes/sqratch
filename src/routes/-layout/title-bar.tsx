import { reactive } from '@legendapp/state/react'
import type { LucideIcon } from 'lucide-react'

import Icons from '#/components/icons'
import Tooltip from '#/components/ui/tooltip'
import { useIsProjectRoute } from '#/lib/hooks/use-route-helpers'
import { cn } from '#/lib/utils'
import BottomPanelStore$ from '#/stores/bottom-panel-store'
import DetailsPanelStore$ from '#/stores/details-panel-store'
import WorkbenchStore$ from '#/stores/workbench-store'

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
				<Icon className={cn(isActive ? 'text-base-content' : 'text-base-content/60', className)} />
			</button>
		</Tooltip>
	),
)

function ProjectActions() {
	return (
		<>
			<ActionButton
				$isActive={WorkbenchStore$.open}
				$label={() => (WorkbenchStore$.open.get() ? 'Hide Left Panel' : 'Show Left Panel')}
				Icon={Icons.PanelLeft}
				onClick={() => WorkbenchStore$.open.toggle()}
			/>
			<ActionButton
				$isActive={BottomPanelStore$.open}
				$label={() => (BottomPanelStore$.open.get() ? 'Hide Bottom Panel' : 'Show Bottom Panel')}
				Icon={Icons.PanelBottom}
				onClick={() => BottomPanelStore$.open.toggle()}
			/>
			<ActionButton
				$isActive={DetailsPanelStore$.open}
				$label={() => (DetailsPanelStore$.open.get() ? 'Hide Right Panel' : 'Show Right Panel')}
				Icon={Icons.PanelRight}
				onClick={() => DetailsPanelStore$.open.toggle()}
			/>
		</>
	)
}
