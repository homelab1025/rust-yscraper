import { useState } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import Sidebar from './components/Sidebar';
import LinkManagementPage from './pages/LinkManagementPage';
import CommentsPage from './pages/CommentsPage';
import AboutPage from './pages/AboutPage';
import LoginPage from './pages/LoginPage';
import { ServicesProvider } from './contexts/ServicesContext';
import AddLinkForm from './components/AddLinkForm';

function AppLayout() {
    const [refreshKey, setRefreshKey] = useState(0);

    return (
        <div className="flex flex-col h-screen overflow-hidden bg-background-light text-slate-900">
            {/* Top header */}
            <header className="flex items-center justify-between border-b border-slate-200 bg-white px-6 py-4 z-10 shrink-0">
                <div className="flex items-center gap-3">
                    <div className="bg-primary p-1.5 rounded text-white flex items-center justify-center">
                        <span className="material-symbols-outlined !text-xl">terminal</span>
                    </div>
                    <h2 className="text-slate-900 text-lg font-bold tracking-tight">What HN is working on</h2>
                </div>
                <AddLinkForm onLinkAdded={() => setRefreshKey(k => k + 1)} />
            </header>

            <div className="flex flex-1 overflow-hidden">
                {/* Sidebar — hidden on mobile */}
                <div className="hidden md:flex shrink-0">
                    <Sidebar />
                </div>

                {/* Main content */}
                <main className="flex-1 overflow-y-auto bg-background-light p-8">
                    <Routes>
                        <Route path="/links" element={<LinkManagementPage key={refreshKey} />} />
                        <Route path="/comments" element={<CommentsPage />} />
                        <Route path="/about" element={<AboutPage />} />
                        <Route path="/" element={<Navigate to="/links" replace />} />
                    </Routes>
                </main>
            </div>
        </div>
    );
}

function App() {
    const [authed, setAuthed] = useState(() => !!localStorage.getItem('auth_token'));

    if (!authed) {
        return <LoginPage onSuccess={() => setAuthed(true)} />;
    }

    return (
        <ServicesProvider>
            <Router>
                <AppLayout />
            </Router>
        </ServicesProvider>
    );
}

export default App;
