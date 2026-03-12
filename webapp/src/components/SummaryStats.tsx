interface SummaryStatsProps {
    reviewed: number;
    picked: number;
    discarded: number;
}

export default function SummaryStats({ reviewed, picked, discarded }: SummaryStatsProps) {
    return (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200 flex flex-col gap-1">
                <div className="flex items-center gap-2 text-slate-500 mb-2">
                    <span className="material-symbols-outlined !text-lg">comment</span>
                    <span className="text-xs font-semibold uppercase tracking-wide">Reviewed Comments</span>
                </div>
                <p className="text-4xl font-black text-slate-900">{reviewed}</p>
            </div>
            <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200 flex flex-col gap-1">
                <div className="flex items-center gap-2 text-emerald-600 mb-2">
                    <span className="material-symbols-outlined !text-lg">check_circle</span>
                    <span className="text-xs font-semibold uppercase tracking-wide">Picked Ideas</span>
                </div>
                <p className="text-4xl font-black text-slate-900">{picked}</p>
            </div>
            <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200 flex flex-col gap-1">
                <div className="flex items-center gap-2 text-rose-600 mb-2">
                    <span className="material-symbols-outlined !text-lg">cancel</span>
                    <span className="text-xs font-semibold uppercase tracking-wide">Discarded Ideas</span>
                </div>
                <p className="text-4xl font-black text-slate-900">{discarded}</p>
            </div>
        </div>
    );
}
