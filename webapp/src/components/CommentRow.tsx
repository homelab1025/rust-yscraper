import * as React from 'react';
import { Button, TableCell, TableRow } from '@mui/material';
import { type CommentDto, CommentState } from '../api-client';

interface CommentRowProps {
    comment: CommentDto;
    selected?: boolean;
    onUpdateState: (commentId: number, state: CommentState) => void;
}

const CommentStateLabels: Record<string, string> = {
    [CommentState.New]: 'New',
    [CommentState.Picked]: 'Picked',
    [CommentState.Discarded]: 'Discarded',
};

const StateColors: Record<string, string> = {
    [CommentState.New]: 'inherit',
    [CommentState.Picked]: '#e8f5e9', // Light green
    [CommentState.Discarded]: '#fafafa', // Very light grey
};

const CommentRow = React.forwardRef<HTMLTableRowElement, CommentRowProps>(
    ({ comment, selected, onUpdateState }, ref) => (
        <TableRow
            ref={ref}
            selected={selected}
            sx={{ backgroundColor: StateColors[comment.state] || 'inherit' }}
        >
            <TableCell>{comment.text}</TableCell>
            <TableCell>{comment.user}</TableCell>
            <TableCell>{comment.subcomment_count}</TableCell>
            <TableCell>{comment.date} ({CommentStateLabels[comment.state]})</TableCell>
            <TableCell>
                <Button
                    variant="text"
                    color="success"
                    disabled={comment.state === CommentState.Picked}
                    onClick={() => onUpdateState(comment.id, CommentState.Picked)}
                >
                    Pick
                </Button>
                <Button
                    variant="text"
                    color="error"
                    disabled={comment.state === CommentState.Discarded}
                    onClick={() => onUpdateState(comment.id, CommentState.Discarded)}
                >
                    Discard
                </Button>
            </TableCell>
        </TableRow>
    )
);

CommentRow.displayName = 'CommentRow';

export default CommentRow;
