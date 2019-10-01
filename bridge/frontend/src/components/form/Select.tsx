import * as React from 'react';
import Select from '@material-ui/core/Select';
import FormControl, { FormControlProps } from '@material-ui/core/FormControl';
import InputLabel from '@material-ui/core/InputLabel';
import FormHelperText, { FormHelperTextProps } from '@material-ui/core/FormHelperText';

import { FieldRenderProps } from 'react-final-form';

interface Props extends FieldRenderProps<string, HTMLElement> {
  label: string;
  formControlProps: FormControlProps;
  FormHelperTextProps?: Partial<FormHelperTextProps>;
  helperText?: React.ReactNode;
}

function FormHelperTextWrapper({
  input: { name, value, onChange, ...restInput },
  meta,
  label,
  helperText,
  FormHelperTextProps,
  formControlProps,
  ...rest
}: Props): React.ReactElement<Props> {
  const showError = ((meta.submitError && !meta.dirtySinceLastSubmit) || meta.error) && meta.touched;
  const labelRef = React.useRef<HTMLLabelElement | null>(null);
  const [labelWidth, setLabelWidth] = React.useState(0);
  React.useEffect(() => {
    labelRef.current && setLabelWidth(labelRef.current.offsetWidth);
  }, [labelRef.current]);

  return (
    <FormControl {...formControlProps} error={showError}>
      <InputLabel ref={labelRef} htmlFor={name}>{label}</InputLabel>

      <Select
        {...rest}
        name={name}
        onChange={onChange}
        inputProps={restInput}
        value={value}
        labelWidth={labelWidth}
      />

      {(showError || helperText) &&
        <FormHelperText {...FormHelperTextProps}>
          {meta.error || meta.submitError || helperText}
        </FormHelperText>
      }
    </FormControl>
  );
}

export default FormHelperTextWrapper;
