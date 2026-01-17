import {
	ArrowsClockwise,
	Database,
	FolderOpen,
	Gear,
	MagnifyingGlass,
	Sparkle,
	Trash
} from '@phosphor-icons/react';
import type {SpaceItem as SpaceItemType} from '@sd/ts-client';
import {useNavigate} from 'react-router-dom';
import {usePlatform} from '../../../contexts/PlatformContext';
import {useLibraryMutation} from '../../../contexts/SpacedriveContext';
import {
	useContextMenu,
	type ContextMenuItem,
	type ContextMenuResult
} from '../../../hooks/useContextMenu';
import {
	getSpaceItemKeyFromRoute,
	useExplorer
} from '../../../routes/explorer/context';
import {
	isLocationItem,
	isPathItem,
	isRawLocation,
	isVolumeItem
} from './spaceItemUtils';

interface UseSpaceItemContextMenuOptions {
	item: SpaceItemType;
	path: string | null;
	spaceId?: string;
}

/**
 * Provides context menu functionality for space items.
 */
export function useSpaceItemContextMenu({
	item,
	path,
	spaceId
}: UseSpaceItemContextMenuOptions): ContextMenuResult {
	const navigate = useNavigate();
	const platform = usePlatform();
	const {loadPreferencesForSpaceItem} = useExplorer();
	const deleteItem = useLibraryMutation('spaces.delete_item');
	const indexVolume = useLibraryMutation('volumes.index');
	const rescanLocation = useLibraryMutation('locations.rescan');
	const removeLocation = useLibraryMutation('locations.remove');

	const items: ContextMenuItem[] = [
		{
			icon: FolderOpen,
			label: 'Open',
			onClick: () => {
				if (path) {
					const [pathname, search] = path.includes('?')
						? [path.split('?')[0], '?' + path.split('?')[1]]
						: [path, ''];
					const spaceItemKey = getSpaceItemKeyFromRoute(
						pathname,
						search
					);
					loadPreferencesForSpaceItem(spaceItemKey);
					navigate(path);
				}
			},
			condition: () => !!path
		},
		{
			icon: Gear,
			label: 'Library Settings',
			onClick: () => {
				navigate('/settings');
			},
			condition: () => (item as any).item_type === 'Library' // Heuristic if applicable
		},
		{
			icon: Database,
			label: 'Index Volume',
			onClick: async () => {
				if (isVolumeItem(item.item_type)) {
					const fingerprint =
						(item as SpaceItemType & {fingerprint?: string})
							.fingerprint || item.item_type.Volume.volume_id;

					try {
						await indexVolume.mutateAsync({
							fingerprint: fingerprint.toString(),
							scope: 'Recursive'
						});
					} catch (err) {
						console.error('Failed to index volume:', err);
					}
				}
			},
			condition: () => isVolumeItem(item.item_type)
		},
		{
			icon: ArrowsClockwise,
			label: 'Quick Rescan',
			onClick: () => {
				const locationId = isLocationItem(item.item_type)
					? item.item_type.Location.location_id
					: (item as any).id;
				if (locationId) {
					rescanLocation.mutate({
						location_id: String(locationId),
						full_rescan: false
					});
				}
			},
			condition: () =>
				isLocationItem(item.item_type) || isRawLocation(item)
		},
		{
			icon: Sparkle,
			label: 'Full Rescan',
			onClick: () => {
				const locationId = isLocationItem(item.item_type)
					? item.item_type.Location.location_id
					: (item as any).id;
				if (locationId) {
					rescanLocation.mutate({
						location_id: String(locationId),
						full_rescan: true
					});
				}
			},
			condition: () =>
				isLocationItem(item.item_type) || isRawLocation(item)
		},
		{type: 'separator'},
		{
			icon: MagnifyingGlass,
			label: 'Show in Finder',
			onClick: async () => {
				if (isPathItem(item.item_type)) {
					const sdPath = item.item_type.Path.sd_path;
					if (typeof sdPath === 'object' && 'Physical' in sdPath) {
						const physicalPath = (
							sdPath as {Physical: {path: string}}
						).Physical.path;
						if (platform.revealFile) {
							try {
								await platform.revealFile(physicalPath);
							} catch (err) {
								console.error('Failed to reveal file:', err);
							}
						}
					}
				}
			},
			keybind: '⌘⇧R',
			condition: () => {
				if (!isPathItem(item.item_type)) return false;
				const sdPath = item.item_type.Path.sd_path;
				return (
					typeof sdPath === 'object' &&
					'Physical' in sdPath &&
					!!platform.revealFile
				);
			}
		},
		{type: 'separator'},
		{
			icon: Trash,
			label: 'Remove Location',
			onClick: async () => {
				const locationId = isLocationItem(item.item_type)
					? item.item_type.Location.location_id
					: (item as any).id;
				if (locationId) {
					try {
						await removeLocation.mutateAsync({
							location_id: String(locationId)
						});
					} catch (err) {
						console.error('Failed to remove location:', err);
					}
				}
			},
			variant: 'danger' as const,
			condition: () =>
				isLocationItem(item.item_type) || isRawLocation(item)
		},
		{
			icon: Trash,
			label: 'Remove from Space',
			onClick: async () => {
				try {
					await deleteItem.mutateAsync({item_id: item.id});
				} catch (err) {
					console.error('Failed to remove item:', err);
				}
			},
			variant: 'danger' as const,
			condition: () =>
				spaceId != null &&
				!isLocationItem(item.item_type) &&
				!isRawLocation(item)
		}
	];

	return useContextMenu({items});
}
