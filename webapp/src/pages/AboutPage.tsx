import * as React from 'react';

export default function AboutPage(): React.JSX.Element {
    return (
        <div className="max-w-3xl mx-auto">
            <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-8">
                <h1 className="text-slate-900 text-2xl font-bold tracking-tight mb-4">About YScraper</h1>
                <p className="text-slate-600 leading-relaxed">
                    YScraper is a tool designed to scrape and manage comments from various sources,
                    starting with Hacker News. It allows you to track specific discussions by
                    adding links and automatically fetching the latest comments for your review.
                </p>
            </div>
        </div>
    );
}
