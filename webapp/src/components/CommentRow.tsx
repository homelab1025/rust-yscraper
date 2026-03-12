import * as React from 'react';
import { type CommentDto, CommentState } from '../api-client';

interface CommentRowProps {
    comment: CommentDto;
    selected?: boolean;
    onUpdateState: (commentId: number, state: CommentState) => void;
}

const CommentRow = React.forwardRef<HTMLTableRowElement, CommentRowProps>(
    ({ comment, selected, onUpdateState }, ref) => {
        const rowClass = [
            'transition-colors',
            comment.state === CommentState.Picked
                ? 'bg-emerald-50 border-l-2 border-emerald-400'
                : comment.state === CommentState.Discarded
                ? 'bg-slate-50 border-l-2 border-slate-300'
                : '',
            selected ? 'bg-primary/10 ring-1 ring-inset ring-primary/20' : '',
        ].filter(Boolean).join(' ');

        return (
            <tr ref={ref} className={rowClass}>
                <td className="px-6 py-4 text-sm text-slate-900 max-w-md">
                    <p className="line-clamp-3">{comment.text}</p>
                </td>
                <td className="px-6 py-4 text-sm text-slate-600 whitespace-nowrap">{comment.user}</td>
                <td className="px-6 py-4 text-sm text-slate-600 text-center">{comment.subcomment_count}</td>
                <td className="px-6 py-4 text-sm text-slate-600 whitespace-nowrap">{comment.date}</td>
                <td className="px-6 py-4">
                    <div className="flex items-center justify-center gap-2">
                        <button
                            onClick={() => onUpdateState(comment.id, CommentState.Picked)}
                            disabled={comment.state === CommentState.Picked}
                            title="Pick"
                            className="text-slate-400 hover:text-emerald-500 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
                        >
                            <span className="material-symbols-outlined !text-xl">check_circle</span>
                        </button>
                        <button
                            onClick={() => onUpdateState(comment.id, CommentState.Discarded)}
                            disabled={comment.state === CommentState.Discarded}
                            title="Discard"
                            className="text-slate-400 hover:text-rose-500 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
                        >
                            <span className="material-symbols-outlined !text-xl">cancel</span>
                        </button>
                    </div>
                </td>
            </tr>
        );
    }
);

CommentRow.displayName = 'CommentRow';

export default CommentRow;
