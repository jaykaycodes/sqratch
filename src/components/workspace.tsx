import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '#/components/ui/resizable'

import StatusBar from '../routes/-project/status-bar'

function Workspace() {
	return (
		<div className="flex grow flex-col">
			<ResizablePanelGroup
				className="max-w-md rounded-lg border md:min-w-[450px]"
				direction="horizontal"
			>
				<ResizablePanel defaultSize={25}>
					<div className="flex h-full items-center justify-center p-6">
						<span className="font-semibold">Sidebar</span>
					</div>
				</ResizablePanel>
				<ResizableHandle withHandle />
				<ResizablePanel defaultSize={75}>
					<div className="flex h-full items-center justify-center p-6">
						<span className="font-semibold">Content</span>
					</div>
				</ResizablePanel>
			</ResizablePanelGroup>

			<StatusBar />
		</div>
	)
}

export default Workspace
