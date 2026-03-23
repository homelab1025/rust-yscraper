import { Link, useLocation } from 'react-router-dom';

export default function Sidebar() {
    const location = useLocation();

    const isActive = (path: string) => location.pathname === path;

    return (
        <aside className="w-80 border-r border-slate-200 bg-white p-6 flex flex-col gap-6 overflow-y-auto">
            <div className="flex flex-col gap-2 border-b border-slate-100 pb-6">
                <Link
                    to="/links"
                    className={`text-sm font-semibold transition-colors flex items-center gap-3 px-2 py-1.5 rounded-lg hover:bg-slate-50 ${
                        isActive('/links')
                            ? 'text-primary bg-slate-50'
                            : 'text-slate-600 hover:text-primary'
                    }`}
                >
                    <span className="material-symbols-outlined !text-lg">link</span>
                    LINKS
                </Link>
                <Link
                    to="/about"
                    className={`text-sm font-semibold transition-colors flex items-center gap-3 px-2 py-1.5 rounded-lg hover:bg-slate-50 ${
                        isActive('/about')
                            ? 'text-primary bg-slate-50'
                            : 'text-slate-600 hover:text-primary'
                    }`}
                >
                    <span className="material-symbols-outlined !text-lg">info</span>
                    ABOUT
                </Link>
            </div>
        </aside>
    );
}
