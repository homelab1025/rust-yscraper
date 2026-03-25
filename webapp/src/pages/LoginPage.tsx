import * as React from 'react';
import { useState } from 'react';

interface LoginPageProps {
    onSuccess: () => void;
}

export default function LoginPage({ onSuccess }: LoginPageProps): React.JSX.Element {
    const [code, setCode] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setLoading(true);
        try {
            const res = await fetch('/api/auth/verify', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ code }),
            });
            if (res.ok) {
                const data = await res.json();
                localStorage.setItem('auth_token', data.token);
                onSuccess();
            } else {
                setError('Invalid code. Please try again.');
            }
        } catch {
            setError('Connection error. Please try again.');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-background-light">
            <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-8 w-full max-w-sm">
                <div className="flex items-center gap-3 mb-6">
                    <div className="bg-primary p-1.5 rounded text-white flex items-center justify-center">
                        <span className="material-symbols-outlined !text-xl">terminal</span>
                    </div>
                    <h2 className="text-slate-900 text-lg font-bold tracking-tight">What HN is working on</h2>
                </div>
                <p className="text-slate-600 text-sm mb-6">Enter the 6-digit code from your Google Authenticator app.</p>
                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <input
                        type="text"
                        inputMode="numeric"
                        pattern="[0-9]{6}"
                        maxLength={6}
                        value={code}
                        onChange={e => setCode(e.target.value)}
                        placeholder="000000"
                        className="border border-slate-300 rounded-lg px-4 py-2 text-center text-2xl tracking-widest font-mono focus:outline-none focus:ring-2 focus:ring-primary"
                        autoFocus
                        required
                    />
                    {error && <p className="text-red-500 text-sm text-center">{error}</p>}
                    <button
                        type="submit"
                        disabled={loading || code.length !== 6}
                        className="bg-primary text-white rounded-lg px-4 py-2 font-medium disabled:opacity-50"
                    >
                        {loading ? 'Verifying…' : 'Login'}
                    </button>
                </form>
            </div>
        </div>
    );
}
