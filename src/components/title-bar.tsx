export default function TitleBar() {
	return (
		<div
			className="h-(--titlebar-height) border-b border-border w-full select-none flex items-center justify-between"
			data-tauri-drag-region
		>
			{/* NOTE: tauri overlays the titlebar with stoplight and title, but we can add buttons at the end  */}
			<div />
			<div>{/* put actions here */}</div>
		</div>
	)
}
