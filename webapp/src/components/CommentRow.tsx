import * as React from 'react';
import { Button, TableCell, TableRow } from '@mui/material';
import { type CommentDto } from '../api-client';

interface CommentRowProps {
    comment: CommentDto;
    selected?: boolean;
}

const CommentRow = React.forwardRef<HTMLTableRowElement, CommentRowProps>(
    ({ comment, selected }, ref) => (
        <TableRow ref={ref} selected={selected}>
            <TableCell>{comment.text}</TableCell>
            <TableCell>{comment.user}</TableCell>
            <TableCell>{comment.date}</TableCell>
            <TableCell>
                <Button variant="text" onClick={() => {}}>Pick</Button>
                <Button variant="text" onClick={() => {}}>Discard</Button>
            </TableCell>
        </TableRow>
    )
);

CommentRow.displayName = 'CommentRow';

export default CommentRow;
