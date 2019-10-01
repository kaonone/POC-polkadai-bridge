import * as React from 'react';
import Checkbox from '@material-ui/core/Checkbox';
import { FieldRenderProps } from 'react-final-form';

type Props = FieldRenderProps<string | number | string[] | undefined, HTMLInputElement>;

function CheckboxWrapper ({
  input: { checked, name, onChange, ...restInput },
  ...rest
}: Props): React.ReactElement<Props> {
  delete rest.meta;

  return (
    <Checkbox
      {...rest}
      name={name}
      inputProps={restInput}
      onChange={onChange}
      checked={checked}
    />
  );
}

export default CheckboxWrapper;
