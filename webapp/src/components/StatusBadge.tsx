interface StatusBadgeProps {
    completed: boolean;
}

export default function StatusBadge({ completed }: StatusBadgeProps) {
    if (completed) {
        return (
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-100 text-emerald-800">
                Completed
            </span>
        );
    }
    return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-800">
            In Progress
        </span>
    );
}
