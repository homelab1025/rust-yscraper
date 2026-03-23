import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';

export default function Sidebar() {
    const location = useLocation();
    const [collapsed, setCollapsed] = useState(false);

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
