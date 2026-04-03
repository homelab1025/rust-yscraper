import { useEffect, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';

export default function Sidebar() {
    const location = useLocation();
    const [collapsed, setCollapsed] = useState(false);
    const [backendInfo, setBackendInfo] = useState<{ git_hash: string; committed_at: string } | null>(null);

    useEffect(() => {
        fetch('/api/info')
            .then(r => r.json())
            .then((data: { git_hash: string; committed_at: string }) => setBackendInfo(data))
            .catch(() => setBackendInfo({ git_hash: 'unknown', committed_at: 'unknown' }));
    }, []);

    const isActive = (path: string) => location.pathname === path;

    return (
        <aside className={`${collapsed ? 'w-16' : 'w-80'} transition-all duration-200 border-r border-slate-200 bg-white flex flex-col overflow-hidden shrink-0`}>
            <div className={`flex flex-col gap-2 border-b border-slate-100 ${collapsed ? 'p-2' : 'p-6'}`}>
                <Link
                    to="/links"
                    title="Links"
                    className={`text-sm font-semibold transition-colors flex items-center py-1.5 rounded-lg hover:bg-slate-50 ${collapsed ? 'justify-center px-2' : 'gap-3 px-2'} ${
                        isActive('/links')
                            ? 'text-primary bg-slate-50'
                            : 'text-slate-600 hover:text-primary'
                    }`}
                >
                    <span className="material-symbols-outlined !text-lg shrink-0">link</span>
                    {!collapsed && <span>LINKS</span>}
                </Link>
                <Link
                    to="/about"
                    title="About"
                    className={`text-sm font-semibold transition-colors flex items-center py-1.5 rounded-lg hover:bg-slate-50 ${collapsed ? 'justify-center px-2' : 'gap-3 px-2'} ${
                        isActive('/about')
                            ? 'text-primary bg-slate-50'
                            : 'text-slate-600 hover:text-primary'
                    }`}
                >
                    <span className="material-symbols-outlined !text-lg shrink-0">info</span>
                    {!collapsed && <span>ABOUT</span>}
                </Link>
            </div>

            <div className="flex-1" />

            {!collapsed && (
                <div className="px-4 pb-2 flex flex-col gap-1">
                    <div className="flex flex-col gap-0.5">
                        <span className="text-[10px] font-mono text-slate-400">fe: {__GIT_HASH__}</span>
                        <span className="text-[10px] font-mono text-slate-300">{__GIT_COMMITTED_AT__}</span>
                    </div>
                    <div className="flex flex-col gap-0.5">
                        <span className="text-[10px] font-mono text-slate-400">be: {backendInfo?.git_hash ?? '…'}</span>
                        <span className="text-[10px] font-mono text-slate-300">{backendInfo?.committed_at ?? ''}</span>
                    </div>
                </div>
            )}

            <div className={`${collapsed ? 'p-2' : 'px-4 pb-4'}`}>
                <button
                    onClick={() => setCollapsed(c => !c)}
                    title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
                    className={`w-full flex items-center rounded-lg text-slate-500 hover:bg-slate-50 hover:text-primary transition-colors ${collapsed ? 'justify-center p-2' : 'gap-3 px-2 py-1.5'}`}
                >
                    <span className="material-symbols-outlined !text-lg shrink-0">
                        {collapsed ? 'chevron_right' : 'chevron_left'}
                    </span>
                    {!collapsed && <span className="text-xs font-semibold">COLLAPSE</span>}
                </button>
            </div>
        </aside>
    );
}
